use std::cell::{Ref, RefMut};
use std::ops::{Deref, DerefMut};

use rapier3d_f64::prelude as rp;

use crate::physics::*;
use crate::rf::Rf;

pub enum RefOrBorrow<'a, T> {
    Ref(Ref<'a, T>),
    Borrow(&'a T),
}

impl<T> Deref for RefOrBorrow<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        match self {
            RefOrBorrow::Ref(r) => &**r,
            RefOrBorrow::Borrow(b) => *b,
        }
    }
}

pub enum RefMutOrBorrow<'a, T> {
    Ref(RefMut<'a, T>),
    Borrow(&'a mut T),
}

impl<T> Deref for RefMutOrBorrow<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        match self {
            RefMutOrBorrow::Ref(r) => &**r,
            RefMutOrBorrow::Borrow(b) => *b,
        }
    }
}

impl<T> DerefMut for RefMutOrBorrow<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        match self {
            RefMutOrBorrow::Ref(r) => &mut **r,
            RefMutOrBorrow::Borrow(b) => *b,
        }
    }
}

/// A wrapper over two states:
/// - A rapier object that does not belong to a world
/// - A rapier object handle that has been added to a world
///
/// This wrapper allows us to access the underlying rapier type regardless of
/// whether it's been added to the world or not.
pub(crate) enum RapierWrapper<H: RapierHandle> {
    Removed(H::Object),
    Added(H, Rf<PhysicsWorld>),
}

impl<H: RapierHandle> RapierWrapper<H> {
    pub(crate) fn as_ref(&self) -> RefOrBorrow<'_, H::Object> {
        match self {
            RapierWrapper::Removed(t) => RefOrBorrow::Borrow(t),
            RapierWrapper::Added(handle, world) => {
                RefOrBorrow::Ref(Ref::map(world.as_ref(), |w| w.get(*handle)))
            }
        }
    }

    pub(crate) fn as_mut(&mut self) -> RefMutOrBorrow<'_, H::Object> {
        match self {
            RapierWrapper::Removed(t) => RefMutOrBorrow::Borrow(t),
            RapierWrapper::Added(handle, world) => {
                RefMutOrBorrow::Ref(RefMut::map(world.as_mut(), |w| w.get_mut(*handle)))
            }
        }
    }

    pub(crate) fn replace(&mut self) -> Self {
        std::mem::replace(
            self,
            RapierWrapper::Added(
                H::invalid(),
                Rf::new(PhysicsWorld {
                    island_manager: rp::IslandManager::new(),
                    rigid_bodies: rp::RigidBodySet::new(),
                    colliders: rp::ColliderSet::new(),
                    narrow_phase: rp::NarrowPhase::new(),
                }),
            ),
        )
    }

    #[allow(dead_code)]
    pub(crate) fn removed_as_ref(&self) -> Option<&H::Object> {
        match self {
            RapierWrapper::Removed(collider) => Some(collider),
            _ => None,
        }
    }

    pub(crate) fn added_as_ref(&self) -> Option<(&H, &Rf<PhysicsWorld>)> {
        match self {
            RapierWrapper::Added(handle, world) => Some((handle, world)),
            _ => None,
        }
    }

    pub(crate) fn set_added<F>(&mut self, world: Rf<PhysicsWorld>, f: F) -> H
    where
        F: FnOnce(H::Object, &mut PhysicsWorld) -> H,
    {
        #[allow(unused_assignments)]
        let mut out_handle = H::invalid();
        *self = match self.replace() {
            RapierWrapper::Removed(collider) => {
                let handle = f(collider, &mut *world.as_mut());
                out_handle = handle;
                RapierWrapper::Added(handle, world)
            }
            RapierWrapper::Added(handle, world) => {
                out_handle = handle;
                RapierWrapper::Added(handle, world)
            }
        };
        out_handle
    }

    pub(crate) fn set_removed<F>(&mut self, f: F)
    where
        F: FnOnce(H, &mut PhysicsWorld) -> H::Object,
    {
        *self = match self.replace() {
            RapierWrapper::Removed(collider) => RapierWrapper::Removed(collider),
            RapierWrapper::Added(handle, world) => {
                RapierWrapper::Removed(f(handle, &mut *world.as_mut()))
            }
        }
    }

    pub(crate) fn is_added(&self) -> bool {
        match self {
            RapierWrapper::Removed(..) => false,
            RapierWrapper::Added(..) => true,
        }
    }

    pub(crate) fn is_removed(&self) -> bool {
        match self {
            RapierWrapper::Removed(..) => true,
            RapierWrapper::Added(..) => false,
        }
    }
}

pub(crate) type RigidBodyWrapper = RapierWrapper<rp::RigidBodyHandle>;
pub(crate) type ColliderWrapper = RapierWrapper<rp::ColliderHandle>;
