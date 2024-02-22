use crate::{ContainerDecoder, Section};
use std::io::Cursor;

fn manual_encode() -> Vec<u8> {
	vec![
		0x57, 0x49, 0x4C, 0x44, // identifier
		0x01, // sections.len()
		0x00, // 0: id
		0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0E, // 1: addr
		0x00, 0x00, 0x00, 0x00, // flags
		0x00, 0x00, 0x00, 0x00, // checksum
		0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // body.len()
	]
}

#[test]
fn encode() {
	let section = Section {
		id: 0,
		flags: 0,
		checksum: 0,
		body: vec![],
	};

	let mut encoded = Cursor::new(vec![]);
	crate::encode(&mut encoded, vec![section]).expect("should not error");

	assert_eq!(encoded.into_inner(), manual_encode());
}

#[test]
fn decode() {
	let encoded = Cursor::new(manual_encode());

	let mut decoder = ContainerDecoder::new(encoded).expect("should not error");
	let section = decoder.next().expect("should exist");

	assert_eq!(
		section,
		Section {
			id: 0,
			flags: 0,
			checksum: 0,
			body: vec![],
		}
	)
}
