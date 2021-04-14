use macroquad::prelude::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

// EXAMPLE IMPLEMENTATION OF RESOURCES
// #[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
// pub enum ExampleIdentifier {
//     Horse,
//     Moose,
// }
//
// pub struct TextureResources {
//     horse: Texture2D,
//     moose: Texture2D,
// }
//
// impl Resources<ExampleIdentifier, Texture2D, DefaultFactory> for TextureResources {
//     fn build(builder: &mut ResourceBuilder<ExampleIdentifier, Self, Texture2D, DefaultFactory>) -> Self {
//         Self {
//             horse: builder.get_or_panic(ExampleIdentifier::Horse),
//             moose: builder.get_or_panic(ExampleIdentifier::Moose),
//         }
//     }
// }

// ready to use resource implementations
pub struct DefaultFactory;
impl ResourceFactory<Texture2D> for DefaultFactory {
    fn load_resource(path: &str) -> Texture2D {
        futures::executor::block_on(load_texture(path))
    }
}

pub trait ResourceFactory<ResourceType> {
    fn load_resource(path: &str) -> ResourceType;
}

// TextureIdentifier: used as a key to acces the resource
pub trait Resources<ResourceIdentifier, ResourceType, F>: Sized
where
    ResourceIdentifier: Eq + Hash + Copy + Clone + Debug,
    F: ResourceFactory<ResourceType>,
{
    fn build(builder: &mut ResourceBuilder<ResourceIdentifier, Self, ResourceType, F>) -> Self;
}

// R: resources
pub struct ResourceBuilder<ResourceIdentifier, R, ResourceType, F>
where
    ResourceIdentifier: Eq + Hash + Copy + Clone + Debug,
    R: Resources<ResourceIdentifier, ResourceType, F> + Sized,
    F: ResourceFactory<ResourceType>,
{
    // path to file
    queued_resources: Vec<(ResourceIdentifier, &'static str)>,
    loaded_resources: HashMap<ResourceIdentifier, ResourceType>,
    total_resources_to_load: i32,
    phantom_resource_r: PhantomData<R>,
    phantom_resource_f: PhantomData<F>,
}

impl<TextureIdentifier, R, ResourceType, F> ResourceBuilder<TextureIdentifier, R, ResourceType, F>
where
    TextureIdentifier: Eq + Hash + Copy + Clone + Debug,
    R: Resources<TextureIdentifier, ResourceType, F>,
    F: ResourceFactory<ResourceType>,
{
    pub fn new(queued_resources: Vec<(TextureIdentifier, &'static str)>) -> Self {
        let total_resources_to_load = queued_resources.len() as i32;
        Self {
            queued_resources,
            loaded_resources: HashMap::new(),
            total_resources_to_load,
            phantom_resource_r: PhantomData,
            phantom_resource_f: PhantomData,
        }
    }

    pub async fn load_next(&mut self) -> bool {
        let is_done = match self.queued_resources.get(0) {
            Some(identifier_name_pair) => {
                let resource = F::load_resource(identifier_name_pair.1); //load_texture(identifier_name_pair.1).await;
                self.loaded_resources
                    .insert(identifier_name_pair.0, resource);
                false
            }
            None => true,
        };
        match is_done {
            false => {
                let _ = self.queued_resources.remove(0);
            }
            true => {}
        }
        is_done
    }

    pub fn progress(&mut self) -> f32 {
        if self.queued_resources.is_empty() {
            1f32
        } else {
            1. - self.queued_resources.len() as f32 / self.total_resources_to_load as f32
        }
    }

    pub fn get_or_panic(&mut self, key: TextureIdentifier) -> ResourceType {
        self.loaded_resources
            .remove(&key)
            .unwrap_or_else(|| panic!("can't find resource: {:?}", key))
    }

    pub fn build(&mut self) -> R {
        R::build(self)
    }
}
