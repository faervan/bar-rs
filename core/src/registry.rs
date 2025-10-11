use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

use merge::Merge as _;

use crate::{config::style::ContainerStyleOverride, module::Module};

pub trait Builder: Any {
    type Output;
    fn build() -> Self::Output;
}

#[allow(clippy::type_complexity)]
#[derive(Default, Debug)]
pub struct Registry {
    modules: HashMap<TypeId, Box<dyn Module>>,
    module_variants: HashMap<String, TypeId>,
    variant_styles: HashMap<String, ContainerStyleOverride>,
    resolvers: HashMap<String, fn() -> Option<(TypeId, String)>>,
}

impl Registry {
    pub fn register_module<T: Builder>(&mut self)
    where
        T::Output: Module,
    {
        let output = T::build();
        let type_id = TypeId::of::<T>();
        for name in output.variant_names() {
            self.module_variants.insert(name.to_string(), type_id);
            self.variant_styles
                .insert(name.to_string(), output.default_style(name));
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

    pub fn get_module<T: Module>(&self) -> &T {
        self.try_get_module().unwrap()
    }

    pub fn get_module_mut<T: Module>(&mut self) -> &mut T {
        self.try_get_module_mut().unwrap()
    }

    pub fn module_by_name_mut(&mut self, name: &String) -> Option<(&TypeId, &mut Box<dyn Module>)> {
        self.module_variants
            .get(name)
            .and_then(|id| self.modules.get_mut(id).map(|m| (id, m)))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn Module>> {
        self.modules.values_mut()
    }

    pub fn iter_enabled<'a, 'b, I>(
        &'a self,
        enabled: I,
    ) -> impl Iterator<Item = (&'b String, &'a Box<dyn Module>, &'a ContainerStyleOverride)>
    where
        I: Iterator<Item = &'b String>,
    {
        enabled.filter_map(|variant| {
            self.module_variants
                .get(variant)
                .copied()
                .or_else(|| {
                    self.resolvers
                        .get(variant)
                        .and_then(|f| f().and_then(|(id, v)| (v == *variant).then_some(id)))
                })
                .and_then(|id| self.modules.get(&id))
                .map(|module| (variant, module, &self.variant_styles[variant]))
        })
    }

    pub fn iter_enabled_mut<'a, I>(
        &'a mut self,
        enabled: I,
    ) -> impl Iterator<Item = &'a mut Box<dyn Module>>
    where
        I: Iterator<Item = &'a str>,
    {
        let resolver_types = self
            .resolvers
            .values()
            .filter_map(|r| r().map(|(id, _)| id))
            .collect::<Vec<TypeId>>();
        let type_ids = self
            .module_variants
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect::<HashMap<&str, &TypeId>>();
        let enabled: HashSet<&str> = enabled.collect();
        self.modules.values_mut().filter(move |m| {
            let names = m.variant_names();
            names.into_iter().any(|name| {
                enabled.contains(name)
                    // TODO: explain why this is needed (if it is needed)
                    || type_ids
                        .get(name)
                        .is_some_and(|ty| resolver_types.contains(*ty))
            })
        })
    }

    pub fn add_resolver<S: ToString>(&mut self, name: S, f: fn() -> Option<(TypeId, String)>) {
        self.resolvers.insert(name.to_string(), f);
    }

    pub fn module_names(&self) -> impl Iterator<Item = &String> {
        self.module_variants.keys()
    }

    pub fn add_module_names(&mut self, type_id: TypeId, names: impl Iterator<Item = String>) {
        let module = &self.modules[&type_id];
        for variant in names {
            let style = module.default_style(&variant);
            self.module_variants.insert(variant.clone(), type_id);
            self.variant_styles.insert(variant, style);
        }
    }

    pub fn remove_module_names(&mut self, names: impl Iterator<Item = String>) {
        for name in names {
            self.module_variants.remove(&name);
            self.variant_styles.remove(&name);
        }
    }

    pub fn set_style_override(
        &mut self,
        type_id: &TypeId,
        variant: &str,
        mut style: ContainerStyleOverride,
    ) {
        let default = self.modules[type_id].default_style(variant);
        style.merge(default);
        *self.variant_styles.get_mut(variant).unwrap() = style;
    }
}
