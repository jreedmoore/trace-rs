use crate::surface::{CanHit, AABB};

pub fn new<'m>(surfaces: &mut [CanHit<'m>]) -> CanHit<'m> {
    let mut bounding = AABB::zero();
    for surface in surfaces.iter() {
        bounding.union_mut(&surface.aabb());
    }
    if surfaces.len() >= 2 {
        let axis = bounding.max_axis();

        surfaces.sort_unstable_by(|a, b| {
            a.aabb().midpoint()[axis]
                .partial_cmp(&b.aabb().midpoint()[axis])
                .unwrap()
        });
        let (mut a, mut b) = surfaces.split_at_mut(surfaces.len() / 2);
        CanHit::BVH {
            bounding,
            children: vec![new(&mut a), new(&mut b)],
        }
    } else {
        CanHit::BVH {
            bounding,
            children: surfaces.to_vec(),
        }
    }
}
/*
pub enum BVH<'a> {
    Internal {
        surfaces: Vec<BVH<'a>>,
        bounding: AABB,
    },
    Leaf {
        surfaces: &'a mut [Box<dyn CanHit + Sync>],
        bounding: AABB,
    }
}
impl<'a> BVH<'a> {
    pub fn new(surfaces: &'a mut [Box<dyn CanHit + Sync>]) -> BVH<'a> {
        let mut bounding = AABB::zero();
        for surface in surfaces.iter() {
            bounding.union_mut(&surface.aabb());
        }
        if surfaces.len() >= 2 {
            let axis = bounding.max_axis();

            surfaces.sort_unstable_by(|a,b| a.aabb().midpoint()[axis].partial_cmp(&b.aabb().midpoint()[axis]).unwrap());
            let (a, b) = surfaces.split_at_mut(surfaces.len() / 2);
            // compute AABB
            // find largest axis
            // sort and split + recurse on this axis
            BVH::Internal {
                surfaces: vec![BVH::new(a), BVH::new(b)],
                bounding: bounding
            }
        } else {
            BVH::Leaf {
                surfaces: surfaces,
                bounding: bounding
            }
        }
    }

    fn bounding(&self) -> &AABB {
        match self {
            BVH::Internal { bounding, .. } => bounding,
            BVH::Leaf { bounding, .. } => bounding,
        }
    }

}
impl<'a> CanHit for BVH<'a> {
    fn ray_intersect(&self, ray: &crate::Ray) -> Option<(f32, &dyn Geometry)> {
        if !self.bounding().ray_hit(ray) {
            return None
        }
        let mut best_hit: Option<(f32, &dyn Geometry)> = None;
        match self {
            BVH::Internal { surfaces, .. } => {
                for surf in surfaces {
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
            }
            BVH::Leaf { surfaces, .. } => {
                for surf in surfaces.iter() {
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
            }
        }
        best_hit
    }

    fn aabb(&self) -> AABB {
        self.bounding().clone()
    }
}
*/
