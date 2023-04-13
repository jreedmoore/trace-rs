use crate::{
    surface::{CanHit, Geometry, AABB},
    Ray,
};

#[derive(Debug)]
pub struct BVH<'a, 'm> {
    internal: Vec<Internal>,
    leaf: Vec<Leaf>,
    surfaces: &'a mut [CanHit<'m>],
    root: usize,
}
impl<'a, 'm> BVH<'a, 'm> {
    pub fn new(surfaces: &'a mut [CanHit<'m>]) -> BVH<'a, 'm> {
        let mut internal = vec![];
        let mut leaf = vec![];
        let p = BVH::new_recur(surfaces, 0, &mut internal, &mut leaf);
        if let ChildPointer::Internal(idx) = p {
            BVH {
                internal,
                leaf,
                surfaces,
                root: idx,
            }
        } else {
            panic!("bvh construction failed")
        }
    }

    fn new_recur(
        surfaces: &'a mut [CanHit<'m>],
        surface_index: usize,
        internal: &mut Vec<Internal>,
        leaf: &mut Vec<Leaf>,
    ) -> ChildPointer {
        let mut bounding = AABB::zero();
        for surface in surfaces.iter() {
            bounding.union_mut(&surface.aabb());
        }
        if surfaces.len() <= 2 {
            let idx = leaf.len();
            leaf.push(Leaf {
                begin: surface_index,
                length: surfaces.len(),
                bounding,
            });
            ChildPointer::Leaf(idx)
        } else {
            let axis = bounding.max_axis();

            surfaces.sort_unstable_by(|a, b| {
                a.aabb().midpoint()[axis]
                    .partial_cmp(&b.aabb().midpoint()[axis])
                    .unwrap()
            });
            let (mut l, mut r) = surfaces.split_at_mut(surfaces.len() / 2);

            let ll = l.len();
            let ln = BVH::new_recur(&mut l, surface_index, internal, leaf);
            let rn = BVH::new_recur(&mut r, surface_index + ll, internal, leaf);

            let idx = internal.len();
            internal.push(Internal {
                left: ln,
                right: rn,
                bounding,
            });
            ChildPointer::Internal(idx)
        }
    }

    pub fn ray_intersect(&'m self, ray: &Ray) -> Option<(f32, &'a dyn Geometry<'m>)> {
        self.ray_intersect_walk(ray, &ChildPointer::Internal(self.root))
    }

    fn ray_intersect_walk(
        &'m self,
        ray: &Ray,
        p: &ChildPointer,
    ) -> Option<(f32, &'a dyn Geometry<'m>)> {
        match p {
            ChildPointer::Internal(i) => {
                let node = &self.internal[*i];
                if !node.bounding.ray_hit(ray) {
                    return None;
                }
                self.ray_intersect_walk(ray, &node.left)
                    .or_else(|| self.ray_intersect_walk(ray, &node.right))
            }
            ChildPointer::Leaf(l) => {
                let leaf = &self.leaf[*l];
                if !leaf.bounding.ray_hit(ray) {
                    return None;
                }
                let mut best_hit = None;
                for surf in self.surfaces[leaf.begin..(leaf.begin + leaf.length)].iter() {
                    if let Some((t, geom)) = surf.ray_intersect(&ray) {
                        if let Some((prior_t, _)) = best_hit {
                            if t < prior_t {
                                best_hit = Some((t, geom));
                            }
                        } else {
                            best_hit = Some((t, geom));
                        }
                    }
                }
                best_hit
            }
        }
    }

    pub fn hits_any(&'m self, ray: &Ray) -> bool {
        self.hits_any_walk(ray, &ChildPointer::Internal(self.root))
    }

    fn hits_any_walk(&'m self, ray: &Ray, p: &ChildPointer) -> bool {
        match p {
            ChildPointer::Internal(i) => {
                let node = &self.internal[*i];
                if !node.bounding.ray_hit(ray) {
                    return false;
                }
                self.hits_any_walk(ray, &node.left) || self.hits_any_walk(ray, &node.right)
            }
            ChildPointer::Leaf(l) => {
                let leaf = &self.leaf[*l];
                if !leaf.bounding.ray_hit(ray) {
                    return false;
                }
                for surf in self.surfaces[leaf.begin..(leaf.begin + leaf.length)].iter() {
                    if surf.hits_any(ray) {
                        return true;
                    }
                }
                false
            }
        }
    }
}
#[derive(Debug, Clone)]
pub struct Internal {
    left: ChildPointer,
    right: ChildPointer,
    bounding: AABB,
}
#[derive(Debug, Clone)]
pub struct Leaf {
    begin: usize,
    length: usize,
    bounding: AABB,
}

#[derive(Debug, Clone)]
enum ChildPointer {
    Internal(usize),
    Leaf(usize),
}