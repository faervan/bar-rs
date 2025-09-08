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
pub struct Registry {
    modules: HashMap<TypeId, Box<dyn Module>>,
    module_names: HashMap<String, TypeId>,
    resolvers: HashMap<String, fn() -> Option<TypeId>>,
}

impl Registry {
    pub fn register_module<T: Builder>(&mut self)
    where
        T::Output: Module,
    {
        let output = T::build();
        let type_id = TypeId::of::<T>();
        for name in output.variant_names() {
            self.module_names.insert(name.to_string(), type_id);
        }
        self.modules.insert(type_id, Box::new(output));
    }

    pub fn try_get_module<T: Module>(&self) -> Option<&T> {
        let id = &TypeId::of::<T>();
        self.modules.get(id).and_then(|t| t.downcast_ref::<T>())
    }

    pub fn try_get_module_mut<T: Module>(&mut self) -> Option<&mut T> {
        let id = &TypeId::of::<T>();
        self.modules.get_mut(id).and_then(|t| t.downcast_mut::<T>())
    }

    pub fn get_module_by_id(&self, id: TypeId) -> &dyn Module {
        self.modules.get(&id).unwrap().as_ref()
    }

    pub fn get_module<T: Module>(&self) -> &T {
        self.try_get_module().unwrap()
    }

    pub fn get_module_mut<T: Module>(&mut self) -> &mut T {
        self.try_get_module_mut().unwrap()
    }

    pub fn get_modules<'a, 'b, I>(
        &'a self,
        enabled: I,
    ) -> impl Iterator<Item = (&'b String, &'a Box<dyn Module>)>
    where
        I: Iterator<Item = &'b String>,
    {
        enabled.filter_map(|id| {
            self.module_names
                .get(id)
                .copied()
                .or_else(|| self.resolvers.get(id).and_then(|f| f()))
                .and_then(|id| self.modules.get(&id))
                .map(|module| (id, module))
        })
    }

    pub fn get_modules_mut<'a, I>(
        &'a mut self,
        enabled: I,
    ) -> impl Iterator<Item = &'a mut Box<dyn Module>>
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

    pub fn module_names(&self) -> impl Iterator<Item = &String> {
        self.module_names.keys()
    }
}
