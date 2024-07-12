#[cfg(feature = "cli")]
use super::ProbeDepth;
use crate::types::{
	Blob, TileBBox, TileCompression, TileCoord3, TileStream, TilesReaderParameters,
};
#[cfg(feature = "cli")]
use crate::utils::PrettyPrint;
use anyhow::Result;
use async_trait::async_trait;
use futures::lock::Mutex;
use std::{fmt::Debug, sync::Arc};

/// Trait defining the behavior of a tile reader.
#[async_trait]
pub trait TilesReaderTrait: Debug + Send + Sync + Unpin {
	/// Get the name of the reader source, e.g., the filename.
	fn get_name(&self) -> &str;

	/// Get the container name, e.g., versatiles, mbtiles, etc.
	fn get_container_name(&self) -> &str;

	/// Get the reader parameters.
	fn get_parameters(&self) -> &TilesReaderParameters;

	/// Override the tile compression.
	fn override_compression(&mut self, tile_compression: TileCompression);

	/// Get the metadata, always uncompressed.
	fn get_meta(&self) -> Result<Option<Blob>>;

	/// Get tile data for the given coordinate, always compressed and formatted.
	async fn get_tile_data(&self, coord: &TileCoord3) -> Result<Option<Blob>>;

	/// Get a stream of tiles within the bounding box.
	async fn get_bbox_tile_stream(&self, bbox: TileBBox) -> TileStream {
		let mutex = Arc::new(Mutex::new(self));
		let coords: Vec<TileCoord3> = bbox.iter_coords().collect();
		TileStream::from_coord_vec_async(coords, move |coord| {
			let mutex = mutex.clone();
			async move {
				mutex
					.lock()
					.await
					.get_tile_data(&coord)
					.await
					.map(|blob_option| blob_option.map(|blob| (coord, blob)))
					.unwrap_or(None)
			}
		})
	}

	/// probe container
	#[cfg(feature = "cli")]
	async fn probe(&mut self, level: ProbeDepth) -> Result<()> {
		use ProbeDepth::*;

		let mut print = PrettyPrint::new();

		let cat = print.get_category("meta_data").await;
		cat.add_key_value("name", self.get_name()).await;
		cat.add_key_value("container", self.get_container_name())
			.await;

		let meta_option = self.get_meta()?;
		if let Some(meta) = meta_option {
			cat.add_key_value("meta", meta.as_str()).await;
		} else {
			cat.add_key_value("meta", &meta_option).await;
		}

		self
			.probe_parameters(&mut print.get_category("parameters").await)
			.await?;

		if matches!(level, Container | Tiles | TileContents) {
			self
				.probe_container(&print.get_category("container").await)
				.await?;
		}

		if matches!(level, Tiles | TileContents) {
			self.probe_tiles(&print.get_category("tiles").await).await?;
		}

		if matches!(level, TileContents) {
			self
				.probe_tile_contents(&print.get_category("tile contents").await)
				.await?;
		}

		Ok(())
	}

	#[cfg(feature = "cli")]
	async fn probe_parameters(&mut self, print: &mut PrettyPrint) -> Result<()> {
		let parameters = self.get_parameters();
		let p = print.get_list("bbox_pyramid").await;
		for level in parameters.bbox_pyramid.iter_levels() {
			p.add_value(level).await
		}
		print
			.add_key_value(
				"bbox",
				&format!("{:?}", parameters.bbox_pyramid.get_geo_bbox()),
			)
			.await;
		print
			.add_key_value("tile compression", &parameters.tile_compression)
			.await;
		print
			.add_key_value("tile format", &parameters.tile_format)
			.await;
		Ok(())
	}

	/// deep probe container
	#[cfg(feature = "cli")]
	async fn probe_container(&mut self, print: &PrettyPrint) -> Result<()> {
		print
			.add_warning("deep container probing is not implemented for this container format")
			.await;
		Ok(())
	}

	/// deep probe container tiles
	#[cfg(feature = "cli")]
	async fn probe_tiles(&mut self, print: &PrettyPrint) -> Result<()> {
		print
			.add_warning("deep tiles probing is not implemented for this container format")
			.await;
		Ok(())
	}

	/// deep probe container tile contents
	#[cfg(feature = "cli")]
	async fn probe_tile_contents(&mut self, print: &PrettyPrint) -> Result<()> {
		print
			.add_warning("deep tile contents probing is not implemented for this container format")
			.await;
		Ok(())
	}

	fn boxed(self) -> Box<dyn TilesReaderTrait>
	where
		Self: Sized + 'static,
	{
		Box::new(self)
	}
}

#[cfg(test)]
mod tests {
	use crate::types::{TileBBoxPyramid, TileFormat};

	use super::*;

	#[derive(Debug)]
	struct TestReader {
		parameters: TilesReaderParameters,
	}

	impl TestReader {
		fn new_dummy() -> TestReader {
			TestReader {
				parameters: TilesReaderParameters {
					bbox_pyramid: TileBBoxPyramid::new_full(3),
					tile_compression: TileCompression::Gzip,
					tile_format: TileFormat::PBF,
				},
			}
		}
	}

	#[async_trait]
	impl TilesReaderTrait for TestReader {
		fn get_name(&self) -> &str {
			"dummy"
		}

		fn get_container_name(&self) -> &str {
			"test container name"
		}

		fn get_parameters(&self) -> &TilesReaderParameters {
			&self.parameters
		}

		fn override_compression(&mut self, tile_compression: TileCompression) {
			self.parameters.tile_compression = tile_compression;
		}

		fn get_meta(&self) -> Result<Option<Blob>> {
			Ok(Some(Blob::from("test metadata")))
		}

		async fn get_tile_data(&self, _coord: &TileCoord3) -> Result<Option<Blob>> {
			Ok(Some(Blob::from("test tile data")))
		}
	}

	#[tokio::test]
	async fn get_bbox_tile_iter() -> Result<()> {
		let reader = TestReader::new_dummy();
		let bbox = TileBBox::new(4, 0, 1, 9, 10)?;
		let stream = reader.get_bbox_tile_stream(bbox).await;

		assert_eq!(stream.drain_and_count().await, 100);

		Ok(())
	}
}
