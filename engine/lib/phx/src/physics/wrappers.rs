use crate::physics::{PhysicsWorld, PhysicsWorldHandle};
use rapier3d::prelude as rp;
use std::cell::{Ref, RefCell, RefMut};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

// struct PhysicsWorldRef<'a> {
//     rc: Rc<RefCell<PhysicsWorld>>,
//     world_ref: Option<Ref<'a, PhysicsWorld>>,
// }

// impl PhysicsWorldRef<'_> {
//     pub fn upgrade<'a>(world: &'a PhysicsWorldHandle) -> PhysicsWorldRef<'a> {
//         let mut r = PhysicsWorldRef {
//             rc: world.upgrade(),
//             world_ref: None,
//         };
//         r.world_ref = Some(r.rc.borrow());
//         r
//     }
// }

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

// Immutable reference to a rapier object, created either from a normal ref or a function.
// pub(crate) struct RapierRef<'a, T>(RefOrBorrow<'a, T>);

// impl<T> RapierRef<'_, T> {
//     fn from_ref(r: &T) -> RapierRef<T> {
//         RapierRef(RefOrBorrow::Borrow(r))
//     }

//     fn from_func<F>(world: &Rc<RefCell<PhysicsWorld>>, get_ref_func: F) -> RapierRef<T>
//     where
//         F: FnOnce(&PhysicsWorld) -> &T
//     {
//         RapierRef(RefOrBorrow::Ref(Ref::map(world.borrow(), get_ref_func)))
//     }
// }

// impl<T> Deref for RapierRef<'_, T> {
//     type Target = T;

//     #[inline]
//     fn deref(&self) -> &T {
//         self.0.deref()
//     }
// }

// A wrapper over either a rigid body or a rigid body handle.
pub(crate) enum RigidBodyWrapper {
    Removed(rp::RigidBody),
    Added(rp::RigidBodyHandle),
}

impl RigidBodyWrapper {
    pub(crate) fn get<'a>(
        &'a self,
        world: &'a Rc<RefCell<PhysicsWorld>>,
    ) -> RefOrBorrow<'a, rp::RigidBody> {
        match self {
            RigidBodyWrapper::Removed(body) => RefOrBorrow::Borrow(body),
            RigidBodyWrapper::Added(handle) => {
                RefOrBorrow::Ref(Ref::map(world.borrow(), |w| w.get_rigid_body(*handle)))
            }
        }
    }

    pub(crate) fn get_mut<'a>(
        &'a mut self,
        world: &'a Rc<RefCell<PhysicsWorld>>,
    ) -> RefMutOrBorrow<'a, rp::RigidBody> {
        match self {
            RigidBodyWrapper::Removed(body) => RefMutOrBorrow::Borrow(body),
            RigidBodyWrapper::Added(handle) => {
                RefMutOrBorrow::Ref(RefMut::map(world.borrow_mut(), |w| {
                    w.get_rigid_body_mut(*handle)
                }))
            }
        }
    }
}

// A wrapper over either a collider or a collider handle.
pub(crate) enum ColliderWrapper {
    Removed(rp::Collider),
    Added(rp::ColliderHandle),
}

impl ColliderWrapper {
    pub(crate) fn get<'a>(
        &'a self,
        world: &'a Rc<RefCell<PhysicsWorld>>,
    ) -> RefOrBorrow<'a, rp::Collider> {
        match self {
            ColliderWrapper::Removed(body) => RefOrBorrow::Borrow(body),
            ColliderWrapper::Added(handle) => {
                RefOrBorrow::Ref(Ref::map(world.borrow(), |w| w.get_collider(*handle)))
            }
        }
    }

    pub(crate) fn get_mut<'a>(
        &'a mut self,
        world: &'a Rc<RefCell<PhysicsWorld>>,
    ) -> RefMutOrBorrow<'a, rp::Collider> {
        match self {
            ColliderWrapper::Removed(body) => RefMutOrBorrow::Borrow(body),
            ColliderWrapper::Added(handle) => {
                RefMutOrBorrow::Ref(RefMut::map(world.borrow_mut(), |w| {
                    w.get_collider_mut(*handle)
                }))
            }
        }
    }
}
