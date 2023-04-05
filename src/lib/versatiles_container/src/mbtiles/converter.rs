use crate::{TileConverterBox, TileConverterTrait, TileReaderBox};
use async_trait::async_trait;
use std::path::Path;
use versatiles_shared::TileConverterConfig;

pub struct TileConverter;

#[async_trait]
impl TileConverterTrait for TileConverter {
	fn new(_filename: &Path, _config: TileConverterConfig) -> TileConverterBox
	where
		Self: Sized,
	{
		panic!("conversion to mbtiles is not supported")
	}
	async fn convert_from(&mut self, _reader: &mut TileReaderBox) {
		panic!("conversion to mbtiles is not supported")
	}
}

#[cfg(test)]
mod tests {
	use super::TileConverter;
	use crate::{dummy, TileConverterTrait, TileReaderTrait};
	use futures::executor::block_on;
	use std::path::Path;
	use versatiles_shared::TileConverterConfig;

	#[test]
	#[should_panic]
	fn test1() {
		let _converter = TileConverter::new(Path::new("filename.txt"), TileConverterConfig::empty());
	}

	#[test]
	#[should_panic]
	fn test2() {
		let mut converter = TileConverter {};
		let mut reader = block_on(dummy::TileReader::new("filename.txt")).unwrap();
		block_on(converter.convert_from(&mut reader));
	}
}
