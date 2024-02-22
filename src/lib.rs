#[cfg(test)]
mod test;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::string::FromUtf8Error;
use thiserror::Error;

const IDENTIFIER: u32 = 0x57494C44;

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Debug, Default, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Section {
	pub id: u8,
	pub flags: u32,
	pub checksum: u32,
	pub body: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum Error {
	#[error("invalid identifier: {0:#X}. required: 0x57494C44")]
	InvalidIdentifier(u32),
	#[error("read error occurred")]
	Io(#[from] io::Error),
	#[error("string was not UTF-8")]
	NotUtf8(#[from] FromUtf8Error),
}

const HEADER_IDENTIFIER: usize = 4;
const HEADER_SECTIONS_LEN: usize = 1;
const HEADER_SECTIONS: usize = 1 + 8;

const SECTION_FLAGS: usize = 4;
const SECTION_CHECKSUM: usize = 4;
const SECTION_BODY_LEN: usize = 8;

pub fn encode(writer: &mut impl Write, sections: Vec<Section>) -> Result<(), Error> {
	writer.write_u32::<BigEndian>(IDENTIFIER)?;

	let sections_len = sections.len() as u8;
	writer.write_u8(sections_len)?;

	let header_offset =
		HEADER_IDENTIFIER + HEADER_SECTIONS_LEN + (sections.len() * HEADER_SECTIONS);

	let mut encoded_sections = Cursor::new(vec![]);

	for idx in 0..sections_len {
		let section = &sections[idx as usize];
		let section_offset =
			SECTION_FLAGS + SECTION_CHECKSUM + SECTION_BODY_LEN + section.body.len();
		let offset = header_offset + (section_offset * idx as usize);

		writer.write_u8(section.id)?;
		writer.write_u64::<BigEndian>(offset as u64)?;

		encoded_sections.write_u32::<BigEndian>(section.flags)?;
		encoded_sections.write_u32::<BigEndian>(section.checksum)?;
		encoded_sections.write_u64::<BigEndian>(section.body.len() as u64)?;
		encoded_sections.write_all(&section.body)?;
	}

	writer.write_all(&encoded_sections.into_inner())?;

	Ok(())
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Default, Debug, Hash)]
pub struct ContainerDecoder<R: Read> {
	reader: R,
	idx: usize,
	pub sections: Vec<(u8, u64)>,
}

impl<R: Read> ContainerDecoder<R> {
	pub fn new(mut reader: R) -> Result<Self, Error> {
		let identifier = reader.read_u32::<BigEndian>()?;
		if identifier != IDENTIFIER {
			return Err(Error::InvalidIdentifier(identifier));
		}

		let sections_len = reader.read_u8()?;
		let mut sections = vec![];

		for _ in 0..sections_len {
			let id = reader.read_u8()?;
			let addr = reader.read_u64::<BigEndian>()?;
			sections.push((id, addr));
		}

		Ok(Self {
			reader,
			idx: 0,
			sections,
		})
	}
}

impl<R: Read + Seek> Iterator for ContainerDecoder<R> {
	type Item = Section;

	fn next(&mut self) -> Option<Self::Item> {
		if self.idx == self.sections.len() {
			return None;
		}

		let (id, addr) = self.sections[self.idx];
		self.idx += 1;

		self.reader.seek(SeekFrom::Start(addr)).ok()?;

		let flags = self.reader.read_u32::<BigEndian>().ok()?;
		let checksum = self.reader.read_u32::<BigEndian>().ok()?;
		let body_len = self.reader.read_u64::<BigEndian>().ok()?;

		let mut body = vec![0; body_len as usize];
		self.reader.read_exact(&mut body).ok()?;

		Some(Section {
			id,
			flags,
			checksum,
			body,
		})
	}
}
