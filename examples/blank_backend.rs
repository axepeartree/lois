use lois::{
    backend::blank::BackendBlank,
    batch::TextureBatchOptions,
    graphics::Graphics,
    texture::{TextureFormat, TextureLoadOptions, TextureUsage},
};

fn main() {
    let backend = BackendBlank::new();
    let mut graphics = Graphics::new(backend);
    let texture = graphics
        .load_texture(TextureLoadOptions {
            format: TextureFormat::Bgra8UnormSrgb,
            height: 200,
            width: 200,
            name: Some(""),
            usage: TextureUsage::RenderTarget,
            ..Default::default()
        })
        .unwrap();
    {
        let mut batch = graphics
            .new_batch(TextureBatchOptions {
                target: None,
                texture,
            })
            .unwrap();
        batch.draw(Default::default());
        batch.draw(Default::default());
        batch.draw(Default::default());
        batch.draw(Default::default());
        batch.draw(Default::default());
    }
    let query = graphics.query_texture(texture).unwrap();
    assert_eq!(query.width, 200);
    assert_eq!(query.height, 200);
    graphics.unload_texture(texture);
    assert!(graphics.query_texture(texture).is_none());
}
