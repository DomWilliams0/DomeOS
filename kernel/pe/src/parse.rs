use crate::address::{Address, FileOffset};
use crate::cursor::Cursor;
use crate::error::{PeError, PeResult};
use crate::types::{CoffHeader, OptionalHeader, SectionHeader};

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
        let mut cursor = self.coff_cursor()?;
        cursor.read_reference()
    }

    pub fn optional_header(&self) -> PeResult<&'pe OptionalHeader> {
        let mut cursor = self.coff_cursor()?;
        let coff: &CoffHeader = cursor.read_reference()?;

        let size = match coff.optional_header_size() {
            Some(sz) => sz.get(),
            None => return Err(PeError::NoOptionalHeader),
        };

        let mut header_cursor = cursor.sub_buffer(size as usize)?;
        let optional_header: &OptionalHeader = header_cursor.read_reference()?;
        optional_header
            .image_type()
            .map_err(PeError::UnsupportedImage)?;
        Ok(optional_header)
    }

    pub fn sections(&self) -> PeResult<impl Iterator<Item = PeResult<&'pe SectionHeader>>> {
        let mut cursor = self.coff_cursor()?;
        let coff: &CoffHeader = cursor.read_reference()?;

        let section_count = coff.num_sections;

        // skip optional header
        cursor.skip(coff.optional_header_size().map(|sz| sz.get()).unwrap_or(0) as usize)?;

        Ok((0..section_count).map(move |_| cursor.read_reference::<SectionHeader>()))
    }

    fn coff_cursor(&self) -> PeResult<Cursor<'pe>> {
        Cursor::new(self.buf, self.coff, self.buf.len() - self.coff.into_usize())
    }
}
