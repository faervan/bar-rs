use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use crate::{config::EnabledModules, listeners::Listener, modules::Module, OptionExt};

pub trait Builder: Any {
    type Output;
    fn build() -> Self::Output;
}

#[derive(Default, Debug)]
pub struct Registry {
    modules: HashMap<TypeId, Box<dyn Module>>,
    listeners: HashMap<TypeId, Box<dyn Listener>>,
    module_names: HashMap<String, TypeId>,
}

#[allow(dead_code)]
impl Registry {
    pub fn register_module<T: Builder>(&mut self)
    where
        T::Output: Module,
    {
        let output = T::build();
        let type_id = TypeId::of::<T>();
        self.module_names.insert(output.id(), type_id);
        self.modules.insert(type_id, Box::new(output));
    }

    pub fn register_listener<T: Builder>(&mut self)
    where
        T::Output: Listener,
    {
        let output = T::build();
        self.listeners.insert(TypeId::of::<T>(), Box::new(output));
    }

    pub fn try_get_module<T: Module>(&self) -> Option<&T> {
        let id = &TypeId::of::<T>();
        self.modules.get(id).and_then(|t| t.downcast_ref::<T>())
    }

    pub fn try_get_listener<T: Listener>(&self) -> Option<&T> {
        let id = &TypeId::of::<T>();
        self.listeners.get(id).and_then(|t| t.downcast_ref::<T>())
    }

    pub fn try_get_module_mut<T: Module>(&mut self) -> Option<&mut T> {
        let id = &TypeId::of::<T>();
        self.modules.get_mut(id).and_then(|t| t.downcast_mut::<T>())
    }

    pub fn try_get_listener_mut<T: Listener>(&mut self) -> Option<&mut T> {
        let id = &TypeId::of::<T>();
        self.listeners
            .get_mut(id)
            .and_then(|t| t.downcast_mut::<T>())
    }

    pub fn get_module<T: Module>(&self) -> &T {
        self.try_get_module().unwrap()
    }

    pub fn get_listener<T: Listener>(&self) -> &T {
        self.try_get_listener().unwrap()
    }

    pub fn get_module_mut<T: Module>(&mut self) -> &mut T {
        self.try_get_module_mut().unwrap()
    }

    pub fn get_listener_mut<T: Listener>(&mut self) -> &mut T {
        self.try_get_listener_mut().unwrap()
    }

    pub fn get_modules<'a, I>(&'a self, enabled: I) -> impl Iterator<Item = &'a Box<dyn Module>>
    where
        I: Iterator<Item = &'a String>,
    {
        enabled.filter_map(|id| {
            self.module_names
                .get(id)
                .and_then(|id| self.modules.get(id))
        })
    }

    pub fn get_modules_mut<'a, I>(
        &'a mut self,
        enabled: I,
    ) -> impl Iterator<Item = &'a mut Box<dyn Module>>
    where
        I: Iterator<Item = &'a String>,
    {
        let enabled: HashSet<&String> = enabled.collect();
        self.modules
            .values_mut()
            .filter(move |m| enabled.contains(&m.id()))
    }

    pub fn get_listeners<'a>(
        &'a self,
        enabled: &'a HashSet<TypeId>,
    ) -> impl Iterator<Item = &'a Box<dyn Listener>> {
        enabled
            .iter()
            .map(|id| self.listeners.get(id).expect("Listener was not registered"))
    }

    pub fn enabled_listeners<'a>(
        &'a self,
        modules: &'a EnabledModules,
    ) -> impl Iterator<Item = TypeId> + 'a {
        modules
            .get_all()
            .filter_map(|m| {
                self.module_names
                    .get(m)
                    .map_none(|| {
                        if !m.is_empty() {
                            eprintln!("No Module named {m} is registered")
                        }
                    })
                    .and_then(|m_id| self.modules.get(m_id).map(|m| m.requires()))
            })
            .flat_map(|required| required.into_iter())
    }

    pub fn all_listeners(&self) -> impl Iterator<Item = (&TypeId, &Box<dyn Listener>)> {
        self.listeners.iter()
    }
}
