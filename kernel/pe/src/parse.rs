use crate::address::{Address, FileOffset};
use crate::cursor::Cursor;
use crate::error::{PeError, PeResult};
use crate::types::{
    CoffHeader, DataDirectory, DataDirectoryDescriptor, DataDirectoryType, OptionalHeader,
    SectionDescriptor,
};

pub struct Pe<'pe> {
    buf: &'pe [u8],
    coff: FileOffset,
}

impl<'pe> Pe<'pe> {
    pub fn from_buffer(buffer: &'pe [u8]) -> PeResult<Self> {
        let mut cursor = Cursor::from(buffer);

        // validate PE signature
        let pe_offset = cursor.read_at_offset::<u32>(0x3c)?;
        cursor.read_until(pe_offset as usize)?;
        cursor.read_magic(0x0000_4550_u32, "PE signature")?;

        let coff_offset = cursor.file_position();
        let pe = Pe {
            buf: buffer,
            coff: coff_offset,
        };

        // validate machine type
        pe.coff()?
            .machine_type()
            .map_err(PeError::UnsupportedMachineType)?;

        // validate image type
        match pe.optional_header() {
            Ok(opt) => {
                opt.image_type().map_err(PeError::UnsupportedImage)?;
            }
            Err(PeError::NoOptionalHeader) => {}
            Err(err) => return Err(err),
        };

        Ok(pe)
    }

    pub fn coff(&self) -> PeResult<&'pe CoffHeader> {
        self.coff_n_cursor().map(|(coff, _)| coff)
    }

    pub fn optional_header(&self) -> PeResult<&'pe OptionalHeader> {
        self.optional_header_n_cursor().map(|(header, _, _)| header)
    }

    pub fn sections(&self) -> PeResult<impl Iterator<Item = PeResult<&'pe SectionDescriptor>>> {
        let mut cursor = self.coff_cursor()?;
        let coff: &CoffHeader = cursor.read_reference()?;

        let section_count = coff.num_sections;

        // skip optional header
        cursor.skip(coff.optional_header_size().map(|sz| sz.get()).unwrap_or(0) as usize)?;

        Ok((0..section_count).map(move |_| cursor.read_reference::<SectionDescriptor>()))
    }

    pub fn data_directories(
        &self,
    ) -> PeResult<impl Iterator<Item = PeResult<(DataDirectoryType, &'pe DataDirectoryDescriptor)>>>
    {
        use strum::IntoEnumIterator;

        let (opt_header, _, mut opt_header_cursor) = self.optional_header_n_cursor()?;

        Ok(DataDirectoryType::iter()
            .zip(0..opt_header.number_of_data_directories)
            .filter_map(move |(ty, _)| {
                match opt_header_cursor.read_reference::<DataDirectoryDescriptor>() {
                    Ok(dd) if !dd.valid() => None,
                    Ok(dd) => Some(Ok((ty, dd))),
                    Err(err) => Some(Err(err)),
                }
            }))
    }

    pub fn data_directory<D: DataDirectory>(&self) -> PeResult<&'pe D> {
        let (_, _, mut opt_header_cursor) = self.optional_header_n_cursor()?;

        let ordinal = D::ORDINAL as usize;
        opt_header_cursor.skip(ordinal * core::mem::size_of::<DataDirectoryDescriptor>())?;

        let dd: &DataDirectoryDescriptor = opt_header_cursor.read_reference()?;
        if !dd.valid() {
            return Err(PeError::MissingDataDirectory(D::ORDINAL));
        }

        // TODO resolve DD to section
        todo!("data directory lookup")
    }

    fn coff_cursor(&self) -> PeResult<Cursor<'pe>> {
        Cursor::new(self.buf, self.coff, self.buf.len() - self.coff.into_usize())
    }

    #[inline]
    fn coff_n_cursor(&self) -> PeResult<(&'pe CoffHeader, Cursor<'pe>)> {
        let mut cursor = self.coff_cursor()?;
        cursor.read_reference().map(|coff| (coff, cursor))
    }

    /// (_, PE cursor, optional header cursor)
    #[inline]
    fn optional_header_n_cursor(
        &self,
    ) -> PeResult<(&'pe OptionalHeader, Cursor<'pe>, Cursor<'pe>)> {
        let (coff, mut cursor) = self.coff_n_cursor()?;

        let size = match coff.optional_header_size() {
            Some(sz) => sz.get(),
            None => return Err(PeError::NoOptionalHeader),
        };

        let mut header_cursor = cursor.sub_buffer(size as usize)?;
        Ok((header_cursor.read_reference()?, cursor, header_cursor))
    }
}
