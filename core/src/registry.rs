use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

use crate::module::Module;

pub trait Builder: Any {
    type Output;
    fn build() -> Self::Output;
}

#[derive(Default, Debug)]
pub struct Registry<Message: 'static> {
    modules: HashMap<TypeId, Box<dyn Module<Message>>>,
    module_names: HashMap<String, TypeId>,
    resolvers: HashMap<String, fn() -> Option<TypeId>>,
}

impl<Message> Registry<Message> {
    pub fn register_module<T: Builder>(&mut self)
    where
        T::Output: Module<Message>,
    {
        let output = T::build();
        let type_id = TypeId::of::<T>();
        for name in output.variant_names() {
            self.module_names.insert(name.to_string(), type_id);
        }
        self.modules.insert(type_id, Box::new(output));
    }

    pub fn try_get_module<T: Module<Message>>(&self) -> Option<&T> {
        let id = &TypeId::of::<T>();
        self.modules.get(id).and_then(|t| t.downcast_ref::<T>())
    }

    pub fn try_get_module_mut<T: Module<Message>>(&mut self) -> Option<&mut T> {
        let id = &TypeId::of::<T>();
        self.modules.get_mut(id).and_then(|t| t.downcast_mut::<T>())
    }

    pub fn get_module_by_id(&self, id: TypeId) -> &dyn Module<Message> {
        self.modules.get(&id).unwrap().as_ref()
    }

    pub fn get_module<T: Module<Message>>(&self) -> &T {
        self.try_get_module().unwrap()
    }

    pub fn get_module_mut<T: Module<Message>>(&mut self) -> &mut T {
        self.try_get_module_mut().unwrap()
    }

    pub fn get_modules<'a, I>(
        &'a self,
        enabled: I,
    ) -> impl Iterator<Item = &'a Box<dyn Module<Message>>>
    where
        I: Iterator<Item = &'a String>,
    {
        enabled.filter_map(|id| {
            self.module_names
                .get(id)
                .copied()
                .or_else(|| self.resolvers.get(id).and_then(|f| f()))
                .and_then(|id| self.modules.get(&id))
        })
    }

    pub fn get_modules_mut<'a, I>(
        &'a mut self,
        enabled: I,
    ) -> impl Iterator<Item = &'a mut Box<dyn Module<Message>>>
    where
        I: Iterator<Item = &'a str>,
    {
        let resolver_types = self
            .resolvers
            .values()
            .filter_map(|r| r())
            .collect::<Vec<TypeId>>();
        let type_ids = self
            .module_names
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect::<HashMap<&str, &TypeId>>();
        let enabled: HashSet<&str> = enabled.collect();
        self.modules.values_mut().filter(move |m| {
            let names = m.variant_names();
            names.into_iter().any(|name| {
                enabled.contains(name)
                    || type_ids
                        .get(name)
                        .is_some_and(|ty| resolver_types.contains(*ty))
            })
        })
    }

    pub fn add_resolver<S: ToString>(&mut self, name: S, f: fn() -> Option<TypeId>) {
        self.resolvers.insert(name.to_string(), f);
    }
}
