use crate::meta::header::{ImageAttributes, Header};
use crate::meta::Headers;
use crate::block::BlockIndex;
use crate::image::{Layers, Layer};
use crate::meta::attribute::{TileDescription};
use crate::prelude::{SmallVec};
use crate::image::write::channels::{WritableChannels, ChannelsWriter};


pub trait WritableLayers<'s> {
    fn infer_headers(&self, image_attributes: &ImageAttributes) -> Headers;

    type Writer: LayersWriter;
    fn create_writer(&'s self, headers: &[Header]) -> Self::Writer;
}

pub trait LayersWriter: Sync {
    fn extract_uncompressed_block(&self, headers: &[Header], block: BlockIndex) -> Vec<u8>;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AllLayersWriter</*'a,*/ C> {
    layers: SmallVec<[LayerWriter</*'a,*/ C>; 2]>
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LayerWriter</*'a,*/ C> {
    channels: C, // impl ChannelsWriter
    // attributes: &'a LayerAttributes,
}



impl<'s, C:'s> WritableLayers<'s> for Layers<C> where C: WritableChannels<'s> {
    fn infer_headers(&self, image_attributes: &ImageAttributes) -> Headers {
        self.iter().map(|layer| layer.infer_headers(image_attributes).remove(0)).collect() // TODO no array-vs-first
    }

    type Writer = AllLayersWriter</*'l,*/ C::Writer>;
    fn create_writer(&'s self, headers: &[Header]) -> Self::Writer {
        AllLayersWriter {
            layers: self.iter().zip(headers.chunks_exact(1)) // TODO no array-vs-first
                .map(|(layer, header)| layer.create_writer(header))
                .collect()
        }
    }
}

impl<'s, C: WritableChannels<'s>> WritableLayers<'s> for Layer<C> {
    fn infer_headers(&self, image_attributes: &ImageAttributes) -> Headers {
        let header = Header {
            channels: self.channel_data.infer_channel_list(),
            compression: self.encoding.compression,

            blocks: match self.encoding.blocks {
                crate::image::Blocks::ScanLines => crate::meta::Blocks::ScanLines,
                crate::image::Blocks::Tiles { tile_size, rounding_mode } => {
                    crate::meta::Blocks::Tiles(TileDescription {
                        level_mode: self.channel_data.level_mode(),
                        tile_size, rounding_mode,
                    })
                },
            },

            line_order: self.encoding.line_order,
            layer_size: self.size,
            shared_attributes: image_attributes.clone(),
            own_attributes: self.attributes.clone(),

            deep: false, // TODO deep data
            deep_data_version: None,
            chunk_count: 0,
            max_samples_per_pixel: None,
        };

        smallvec![ header ]// TODO no array-vs-first
    }

    type Writer = LayerWriter</*'l,*/ C::Writer>;
    fn create_writer(&'s self, headers: &[Header]) -> Self::Writer {
        let channels = self.channel_data.create_writer(headers.first().unwrap()); // TODO no array-vs-first

        LayerWriter {
            channels
            // attributes: &self.attributes
        }
    }
}

impl</*'a,*/ C> LayersWriter for AllLayersWriter</*'a,*/ C> where C: ChannelsWriter {
    fn extract_uncompressed_block(&self, headers: &[Header], block: BlockIndex) -> Vec<u8> {
        self.layers[block.layer].extract_uncompressed_block(&headers[block.layer..block.layer], block) // TODO no array-vs-first
    }
}

impl</*'a,*/ C> LayersWriter for LayerWriter</*'a,*/ C> where C: ChannelsWriter {
    fn extract_uncompressed_block(&self, headers: &[Header], block: BlockIndex) -> Vec<u8> {
        self.channels.extract_uncompressed_block(headers.first().unwrap(), block) // TODO no array-vs-first
    }
}

/*pub trait WritableLayers {
    fn generate_meta_data(&self, shared_attributes: &ImageAttributes) -> Headers;
    fn extract_block(&self, headers: &[Header], block: BlockIndex) -> Vec<u8>;
}

impl<C: WritableChannels> WritableLayers for Layers<C> {
    fn generate_meta_data(&self, shared_attributes: &ImageAttributes) -> Headers {
        self.iter().map(|layer| layer.generate_meta_data(shared_attributes).first().unwrap()).collect()
    }

    fn extract_block(&self, headers: &[Header], block: BlockIndex) -> Vec<u8> {
        self.get(block.layer).expect("invalid block index")
            .extract_block(headers.get(block.layer .. block.layer).unwrap(), block)
    }
}*/

/*impl<C: WritableChannels> WritableLayers for Layer<C> {
    fn generate_meta_data(&self, shared_attributes: &ImageAttributes) -> Headers {
        let header = Header {
            channels: ChannelList::new(self.channel_data.generate_meta_data()),
            compression: self.encoding.compression,

            blocks: match self.encoding.blocks {
                crate::image::Blocks::ScanLines => crate::meta::Blocks::ScanLines,
                crate::image::Blocks::Tiles { tile_size, rounding_mode } => {
                    crate::meta::Blocks::Tiles(TileDescription {
                        level_mode: self.channel_data.level_mode(),
                        tile_size, rounding_mode,
                    })
                },
            },

            line_order: self.encoding.line_order,
            layer_size: self.size,
            shared_attributes: shared_attributes.clone(),
            own_attributes: self.attributes.clone(),

            deep: false, // TODO deep data
            deep_data_version: None,
            chunk_count: 0,
            max_samples_per_pixel: None,
        };

        smallvec![ header ]
    }

    fn extract_block(&self, header: &[Header], block: BlockIndex) -> Vec<u8> {
        self.channel_data.extract_block(header.first().unwrap(), block)
    }
}*/
