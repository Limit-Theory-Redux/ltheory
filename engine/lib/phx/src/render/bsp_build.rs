use glam::Vec3;

use crate::math::{Plane, Polygon, PolygonClassification, Rng, lerp};

pub(super) struct BspBuild {
    // pub root_node: Option<BSPBuildNode>,
    pub node_count: usize,
    pub leaf_count: usize,
    pub triangle_count: usize,
    // CHECK2 (
    //     int32 nextNodeID;
    //     int32 oversizedNodes;
    //     float avgOversizeAmount;
    // )
}

#[derive(Clone)]
pub(super) struct BspBuildNode {
    pub plane: Plane,
    pub child: [Option<Box<BspBuildNode>>; 2],
    pub polygons: Vec<PolygonEx>,
    // CHECK2 (
    //     int32 id;
    //     BSPBuild_Node* parent;
    //     Vec3f planeCenter;
    // )
}

impl BspBuildNode {
    fn new() -> Self {
        Self {
            plane: Plane {
                n: Vec3::ZERO,
                d: 0.0,
            },
            child: Default::default(),
            polygons: vec![],
        }
    }
}

#[derive(Clone)]
pub(super) struct PolygonEx {
    pub inner: Polygon,
    pub flags: u8,
}

#[derive(Clone)]
pub(super) struct BspBuildNodeData {
    pub polygons: Vec<PolygonEx>,
    pub valid_polygon_count: usize,
    pub triangle_count: usize,
    pub depth: u16,
    //Box3f boundingBox;
    //uint8 cutIndex;
}

// const DEFAULT_TRIANGLE_SPLIT_COST: f32 = 0.85;
// const LEAF_TRIANGLE_COUNT: i32 = 12;
pub const BACK_INDEX: usize = 0;
pub const FRONT_INDEX: usize = 1;

#[derive(Clone, Copy)]
enum PolygonFlag {
    FaceSplit = 1 << 0,
    Decompose = 1 << 1,
    EdgeSplit = 1 << 2,
}

impl BspBuildNodeData {
    fn score_split_plane(&self, plane: Plane, k: f32) -> f32 {
        /* The bigger k is, the more we penalize polygon splitting */
        // Assert(k >= 0.0f && k <= 1.0f);

        let mut num_in_front = 0;
        let mut num_behind = 0;
        let mut num_straddling = 0;

        for polygon in &self.polygons {
            match plane.classify_polygon(&polygon.inner) {
                PolygonClassification::Coplanar | PolygonClassification::Behind => num_behind += 1,
                PolygonClassification::InFront => num_in_front += 1,
                PolygonClassification::Straddling => num_straddling += 1,
            }
        }

        // k*numStraddling + (1.0f - k)*Abs(numInFront - numBehind);
        lerp(
            f64::abs((num_in_front - num_behind) as f64),
            num_straddling as f64,
            k as f64,
        ) as f32
    }

    #[inline]
    fn append_polygon(&mut self, polygon: &PolygonEx) {
        //if (nodeData->triangleCount == 0) {
        //  Vec3f v0 = ArrayList_Get(polygon->vertices, 0);
        //  nodeData->boundingBox.lower = v0;
        //  nodeData->boundingBox.upper = v0;
        //}
        //ArrayList_ForEach(polygon->vertices, Vec3f, v) {
        //  Box3f_Add(&nodeData->boundingBox, *v);
        //}

        self.triangle_count += polygon.inner.vertices.len() - 2;
        if polygon.flags as usize & PolygonFlag::FaceSplit as usize == 0 {
            self.valid_polygon_count += 1;
        }
        self.polygons.push(polygon.clone())
    }
}

impl BspBuild {
    fn choose_split_plane(
        node_data: &mut BspBuildNodeData,
        split_plane: &mut Plane,
        rng: &mut Rng,
    ) -> bool {
        /* See Realtime Collision Detection pp361-363 */

        /* Misc Notes from the Literature
         *  TODO : The number of candidates c selected at each call as a percentage of the
         *  number of faces f lying in the current region is increased as a linear function
         *  of f until a predetermined threshold is reached, after which all face hyperplanes
         *  are chosen (currently 20).
         *
         *  NOTE: Since we are interested in generating a multiresolution representation,
         *  we bias the selection process by first sorting the face hyperplanes by area
         *  (each hyperplane is represented only once, and has with it a list of coincident
         *  faces). The candidates are then the first c on this sorted list.
         *  https://pdfs.semanticscholar.org/8fa2/b73cb14fad3abe749a0da4fba50f18a19e2a.pdf
         *  This method sucked! Vastly slower than random choices. I only sorted the list
         *  once, not every single split. Perhaps this breaks the algorithm? Either way,
         *  sorting triangles every single split during tree construction is going to
         *  annihilate build time.
         *
         *  TODO : For each of a predefined number of directions, we project all of the
         *  vertices onto that direction and then sort them. We then consider hyperplanes
         *  orthogonal to this direction which contain vertices at certain positions in the
         *  ordering. The percentage of positions tested is treated similarly to that for
         *  choosing the number of face hyperplanes. The directions we are currently using
         *  correspond to the k-faces of a hypercube, whose number in 3D is 13 = 26/2 (see
         *  figure 7 for the 2D case).
         *  https://pdfs.semanticscholar.org/8fa2/b73cb14fad3abe749a0da4fba50f18a19e2a.pdf
         *
         *  TODO : The third method is similar to the second, but uses least squares fit
         *  to generate a direction. In particular, we compute the least squares fit of the
         *  set of vertices lying in the current region, and then use the normal of the
         *  resulting hyperplane as a new direction when applying the same techniques as
         *  used with the predefined directions.
         *  https://pdfs.semanticscholar.org/8fa2/b73cb14fad3abe749a0da4fba50f18a19e2a.pdf
         */

        let max_depth = 1000.0;
        let biased_depth = node_data.depth as f32 - 100.0;
        let t = f64::max((biased_depth / max_depth) as f64, 0.0) as f32;
        let k = lerp(0.85, 0.25, t as f64) as f32;

        let mut best_score = f32::MAX;
        let mut best_plane = Plane {
            n: Vec3::ZERO,
            d: 0.,
        };
        let mut num_to_check = 10;

        let polygons_len = node_data.polygons.len();
        if node_data.valid_polygon_count > 0 {
            /* Simply score split planes using polygon faces */
            let mut best_polygon_index = node_data.polygons.len();
            num_to_check = usize::min(num_to_check, node_data.valid_polygon_count);
            for _ in 0..num_to_check {
                let mut polygon_index = rng.get32().wrapping_rem(polygons_len as u32) as usize;

                /* OPTIMIZE: This search is duuuuuumb. Maybe We should swap invalid
                 *           polygons to the end of the list so never have to search.
                 */
                for _ in 0..polygons_len {
                    if node_data.polygons[polygon_index].flags as u8 & PolygonFlag::FaceSplit as u8
                        == 0
                    {
                        let plane = node_data.polygons[polygon_index].inner.to_plane();
                        let score = node_data.score_split_plane(plane, k);

                        if score < best_score {
                            best_score = score;
                            best_plane = plane;
                            best_polygon_index = polygon_index;
                            // node_data.polygons[polygon_index].flags =
                            //     node_data.polygons[polygon_index].flags as u8
                            //         | PolygonFlag::FaceSplit as u8;
                            // CHECK2(Polygon_GetCentroid((Polygon*) bestPolygon, &node->planeCenter);)
                        }
                        break;
                    }

                    polygon_index = (polygon_index + 1) % polygons_len;
                }
            }
            if best_polygon_index < node_data.polygons.len() {
                node_data.polygons[best_polygon_index].flags =
                    node_data.polygons[best_polygon_index].flags as u8
                        | PolygonFlag::FaceSplit as u8;
            }
        } else if polygons_len > 0 {
            /* No remaining polygons are valid for splitting. So we split any polygons
             * with multiple triangles. When none of those are left, we use the polygon
             * edges as split planes with no penalty for cutting other polygons.
             */

            /* EDGE: It's possible to get to a point where all remaining polygons are
             * invalid for auto partitioning, but there are still more triangles than
             * the max leaf size. In this case we need to start dividing the polygons.
             * If we don't do this, it makes a significant impact on overall tree size
             * because we actually end up with quite a few leaves with more triangles
             * than MAX_LEAF_TRIANGLE_COUNT
             */

            /* EDGE: With very few polygons BSPBuild_ScoreSplitPlane will prioritize 100%
             * lopsided splits over a split with a single cut. This leads to picking
             * the same, useless general cut again next time.
             */

            /* Note that the flags set by these additional splitting steps will be
             * transferred to the resulting pieces if the polygon is ever split. This
             * is currently necessary because if the cut is chosen but this polygon
             * can't be split safely (produces degenerate or tiny polgons, see
             * Polygon_SplitSafe) we create 2 new 'split' polygons that are actually
             * the full polygon and send it to both sides of the plane. We might be
             * able to remove this for slightly better splitting (e.g. ending up with
             * fewer oversized leaves because we tried more cuts) but it needs to be
             * done carefully. */

            let mut split_found = false;

            /* Try to split any polygons with more than 1 triangle */
            if !split_found {
                let mut best_polygon_index = node_data.polygons.len();
                let mut polygon_index = rng.get32().wrapping_rem(polygons_len as u32) as usize;
                for _ in 0..polygons_len {
                    let polygon = &mut node_data.polygons[polygon_index];
                    if polygon.flags as u8 & PolygonFlag::Decompose as u8 != 0 {
                        continue;
                    }

                    let v = &polygon.inner.vertices;
                    for j in 2..(v.len() - 1) {
                        let edge = v[0] - v[j];
                        let mid = Vec3::lerp(v[0], v[j], 0.5f32);

                        /* TODO : Maybe just save the plane with polygon while build so they're only calculated once? */
                        let polygon_plane = polygon.inner.to_plane();
                        let mut plane: Plane = Plane {
                            n: Vec3::ZERO,
                            d: 0.,
                        };
                        plane.n = Vec3::cross(edge, polygon_plane.n).normalize();
                        plane.d = Vec3::dot(plane.n, mid);

                        /* TODO : Proper scoring? */
                        if plane.classify_polygon(&polygon.inner)
                            == PolygonClassification::Straddling
                        {
                            split_found = true;

                            best_score = 0.0;
                            best_plane = plane;
                            best_polygon_index = polygon_index;
                            // polygon.flags = polygon.flags as u8 | PolygonFlag::Decompose as u8;
                            // CHECK2(node->planeCenter = mid;)
                            break;
                        } else {
                            /* This is possible because we don't fully handle slivers. There's
                             * nothing stopping a triangle from being thinner than
                             * PLANE_THICKNESS_EPSILON. */
                            polygon.flags = polygon.flags as u8 | PolygonFlag::Decompose as u8;
                        }
                        //if (--numToCheck == 0) break;
                    }

                    if split_found {
                        break;
                    }
                    //if (numToCheck == 0) break;
                    polygon_index = (polygon_index + 1) % polygons_len;
                }

                if split_found {
                    node_data.polygons[best_polygon_index].flags =
                        node_data.polygons[best_polygon_index].flags as u8
                            | PolygonFlag::Decompose as u8;
                }
            }

            /* Try splitting along a polygon edge */
            if !split_found {
                let mut plane = Plane {
                    n: Vec3 {
                        x: 0.,
                        y: 0.,
                        z: 0.,
                    },
                    d: 0.,
                };
                let mut best_polygon_index = node_data.polygons.len();
                let mut polygon_index = rng.get32().wrapping_rem(polygons_len as u32) as usize;
                for _ in 0..polygons_len {
                    if node_data.polygons[polygon_index].flags as u8 & PolygonFlag::EdgeSplit as u8
                        != 0
                    {
                        continue;
                    }

                    let polygon_plane = node_data.polygons[polygon_index].inner.to_plane();
                    let v = &node_data.polygons[polygon_index].inner.vertices;
                    let mut v_prev = v[v.len() - 1];

                    #[allow(clippy::needless_range_loop)]
                    // Cannot convert into `for_each` because of break instruction
                    for j in 0..v.len() {
                        let v_cur = v[j];
                        let edge = v_cur - v_prev;
                        let mid = Vec3::lerp(v_prev, v_cur, 0.5f32);

                        plane.n = Vec3::cross(edge, polygon_plane.n).normalize();
                        plane.d = Vec3::dot(plane.n, mid);

                        let score = node_data.score_split_plane(plane, 0.0f32);
                        if score < best_score {
                            split_found = true;

                            best_polygon_index = polygon_index;
                            best_score = score;
                            best_plane = plane;
                            // CHECK2(node->planeCenter = mid;)
                        }

                        v_prev = v_cur;
                        num_to_check -= 1;
                        if num_to_check == 0 {
                            break;
                        }
                    }

                    if num_to_check == 0 {
                        break;
                    }
                    polygon_index = (polygon_index + 1) % polygons_len;
                }

                if split_found {
                    node_data.polygons[best_polygon_index].flags =
                        node_data.polygons[best_polygon_index].flags as u8
                            | PolygonFlag::EdgeSplit as u8;
                }
            }

            // CHECK3 (
            //   /* Still nothing. Fuck it. */
            //   if (!splitFound) {
            //     int32 triangleCount = 0;
            //     ArrayList_ForEach(nodeData->polygons, PolygonEx, polygon) {
            //       triangleCount += ArrayList_GetSize(polygon->vertices) - 2;
            //     }
            //     bsp->oversizedNodes++;
            //     float oversizeAmount = (float) (triangleCount - LEAF_TRIANGLE_COUNT);
            //     bsp->avgOversizeAmount = Lerp(bsp->avgOversizeAmount, oversizeAmount, 1.0f / bsp->oversizedNodes);
            //     Warn("BSPBuild_ChooseSplitPlane: Failed to find a good split. Giving up. Leaf will have %i triangles.", triangleCount);
            //   }
            // )
        } else {
            /* We don't have any polygons left. All of them were on the same side of
             * the last split. This will end up being a leaf.  */
        }

        if best_score < f32::MAX {
            *split_plane = best_plane;
            true
        } else {
            false
        }
    }

    pub fn create_node(&mut self, node_data: &mut BspBuildNodeData, rng: &mut Rng) -> BspBuildNode {
        /* NOTE: This will free the polygons being passed in! This is to prevent all
         *        the temporary allocations from overlapping. */

        /* NOTE: Coplanar polygons are considered to be behind the plane and will
         *        therefore lead to collisions. It seems preferable to push objects
         *        very slightly outside of each other during a collision, rather than
         *        letting them very slightly overlap. */

        // Assert(nodeData->depth < 1 << 8*sizeof(nodeData->depth));

        let mut node = BspBuildNode::new();
        // CHECK2(node->id = bsp->nextNodeID++;)

        let mut split_plane = Plane {
            n: Vec3::ZERO,
            d: 0.,
        };

        let make_leaf = node_data.triangle_count <= 12
            || !Self::choose_split_plane(node_data, &mut split_plane, rng);

        if make_leaf {
            if node_data.triangle_count != 0 {
                self.leaf_count += 1;
            }
            self.triangle_count += node_data.triangle_count;

            node.polygons = node_data.polygons.clone();

            return node;
        }

        self.node_count += 1;

        let polygons_len = node_data.polygons.len();

        let mut back_node_data = BspBuildNodeData {
            polygons: Vec::new(),
            valid_polygon_count: 0,
            triangle_count: 0,
            depth: 0,
        };
        back_node_data.polygons.reserve(polygons_len);
        back_node_data.depth = (node_data.depth as i32 + 1) as u16;

        let mut front_node_data = BspBuildNodeData {
            polygons: Vec::new(),
            valid_polygon_count: 0,
            triangle_count: 0,
            depth: 0,
        };
        front_node_data.polygons.reserve(polygons_len);
        front_node_data.depth = (node_data.depth as i32 + 1) as u16;

        for polygon in node_data.polygons.iter_mut() {
            let classification = split_plane.classify_polygon(&polygon.inner);
            match classification {
                PolygonClassification::Coplanar => {
                    polygon.flags = polygon.flags as u8 | PolygonFlag::FaceSplit as u8;
                }
                PolygonClassification::Behind => back_node_data.append_polygon(polygon),
                PolygonClassification::InFront => front_node_data.append_polygon(polygon),
                PolygonClassification::Straddling => {
                    let mut back_part = PolygonEx {
                        inner: Polygon {
                            vertices: Vec::new(),
                        },
                        flags: 0,
                    };
                    back_part.flags = polygon.flags;

                    let mut front_part = PolygonEx {
                        inner: Polygon {
                            vertices: Vec::new(),
                        },
                        flags: 0,
                    };
                    front_part.flags = polygon.flags;

                    polygon.inner.split_safe(
                        &split_plane,
                        &mut back_part.inner,
                        &mut front_part.inner,
                    );
                    back_node_data.append_polygon(&back_part);
                    front_node_data.append_polygon(&front_part);

                    polygon.inner.vertices.clear();
                }
            }
        }
        node_data.polygons.clear();

        node.plane = split_plane;
        node.child[BACK_INDEX] = Some(self.create_node(&mut back_node_data, rng).into());
        node.child[FRONT_INDEX] = Some(self.create_node(&mut front_node_data, rng).into());

        // CHECK2 (
        //     node->child[BackIndex] ->parent = node;
        //     node->child[FrontIndex]->parent = node;
        // )

        node
    }
}
