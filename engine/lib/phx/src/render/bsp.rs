use glam::Vec3;

use super::{
    BACK_INDEX, BspBuild, BspBuildNode, BspBuildNodeData, Color, FRONT_INDEX, Mesh, PolygonEx,
};
use crate::logging::warn;
use crate::math::{Intersect, LineSegment, Plane, Polygon, Position, Ray, Rng, Sphere, Triangle};
use crate::render::{BlendMode, CullFace, Draw, RenderState, Shader};

/* Adam's Stupidly Fast BSP Implementation
 *
 * Implementation Details
 * - We refer to internal tree nodes as 'nodes', and leaf nodes as 'leaves'.
 * - BSP.nodes contains the internal nodes (and *not* any leaf nodes).
 * - BSP.triangles contains all the final (split) triangles.
 * - We don't store leaf nodes at all. Nodes use a negative index to indicate
 *     the target is a leaf. This index is relative to BSP.triangles. The node
 *     also stores a triangleCount.
 * - No leaves are actually stored at all. Instead, nodes store indices to a flat triangle list.
 * - At 4 triangles per leaf, 67% of the tree size is the actual triangles.
 * - At 32 triangles per leaf triangles are 88% of the tree.
 * - Thus, the only ways to significantly reduce tree size are to either choose
 *     splits that cut fewer triangles or store references to the original mesh
 *     vertices (or triangles, but care would have to be taken to handle new
 *     triangles formed by cutting).
 *
 * TODO
 * Build Speed Optimizations
 *  There's not much obvious waste in the build process, so there's not enough
 *  'low-hanging fruit' to reduce the build time by more than 10-20%.
 *  For reference, 98% of the time is spent in in BSPBuild_CreateNode (as
 *  opposed to the prologue or epilogue in BSP_Create). Of that, 72% of the
 *  total time is spend in BSPBuild_ChooseSplitPlane and 24% is spent splitting
 *  polygons to front and back lists. This makes is clear that the only place
 *  worth spending optimization time right now is BSPBuild_ChooseSplitPlane.
 *
 *  To make a major difference in speed consider the following approaches:
 *    1) Stop splitting triangles at all
 *       Short of fundammentally improving the plane selection algorithm, I
 *       think this is the most promising optimization and should be done before
 *       any others.
 *    2) Improve the core plane selection algorithm (do general splits!!).
 *    3) Find a way to make BSPBuild_CreateNode iterative instead of recursive.
 *
 * Once those are d one consider the following smaller optimizations:
 *    1) Early out of ScoreSplittingPlane
 *       This requires deriving a way to predict when the best possible score is
 *       worse than the current best score. This is a constrained, multivariate
 *       optimization problem.
 *    2) Remember the result of Plane_ClassifyPolygon when returning from
 *       ChooseSplitPlane.
 *       I saw an 8% gain by doing this.
 *    3) Store planes with triangles
 *    4) Reuse the incoming polygon list in BSPBuild_CreateNode
 *
 * - Replace BSP_Create with BSP_Create(Polygon* polygons, int32 polygonsLen) and BSP_FromMesh(Mesh*)
 * - Just go back to using positive indices and check triangleCount instead.
 * - Need profiling for the tree build time and max memory used.
 * - Implement hinting planes.
 * - Consider triangles in mesh.
 * - The build process is using a *lot* of temporary memory. This will need to be addressed.
 * - Probably want to use a stack based scratch allocator when building the tree.
 * - Leaf storing or node storing? (Put unsplit triangles in nodes instead of passing to both sides?)
 * - Get rid of the Mesh -> Polygon conversion in BSP_Create?
 * - Store internal nodes like a breadth first search.
 * - Prefetch nodes or refactor to remove data dependency (pack nodes and index into an array)
 * - Make it thread safe
 */

/* Performance Data
 *   (Currently biased for hits. Just getting structure together.)
 *   Each test is run with the 300,000 rays. Rays are the same for individual tests, different when varying a parameter
 * |---------------------------------------------------------------------------------------------------------------------------------------------------|
 * |          |         |      |  BSP  |  MiB  |           |  Tri  |           |         |  Max  | Avg Leaf ||  Avg   |  Avg   |  Avg  |  Avg  | Avg   |
 * | Mesh     |  Tris   | MiB  |  MiB  | Ratio |   Tris    | Ratio |   Nodes   | Leaves  | Depth |  Depth   || Ray us | Leaves | Nodes | Depth | Tris  |
 * |-------------------------------------------------------------------------------------------------------------------------------------------|-------|
 * | luffa    | 327,680 | 30.0 | 119.2 | 3.97  | 1,350,532 | 4.12  | 1,240,357 | 620,179 |  736  |   97.3   ||  21.8  |  25.8  | 301.2 | 107.3 |  60.8 |  Halfway split poly, 1 poly/leaf
 * |          |         |      | 117.0 | 3.90  | 1,286,089 | 3.92  | 1,241,483 | 620,742 |  956  |   41.1   ||  16.2  |  20.5  | 181.3 |  50.4 |  59.1 |  Random split poly,  1 poly/leaf
 * |          |         |      |  89.6 | 2.99  | 1,286,736 | 3.93  |   781,263 | 390,632 |  807  |   45.2   ||  16.4  |  16.8  | 162.0 |  50.5 |  83.8 |  Random split poly,  4 poly/leaf
 * |          |         |      |       |       |           |       |           |         |       |          ||        |        |       |       |       |
 * |          |         |      |  83.2 | 2.77  |   881,460 | 2.69  |   913,521 | 456,761 |  627  |   33.1   ||  13.6  |  17.8  | 151.8 |  35.6 |  45.3 |  Best of 10, k=0.8,  1 poly/leaf
 * |          |         |      |  43.2 | 1.44  |   822,198 | 2.51  |   260,919 | 130,460 |  497  |   49.3   ||  18.7  |  11.2  | 106.5 |  36.6 | 195.2 |  Best of 10, k=0.8, 32 tris/leaf
 * |          |         |      |  49.5 | 1.65  |   858,619 | 2.62  |   349,793 | 174,897 |  395  |   40.3   ||  16.6  |  12.4  | 115.1 |  33.5 | 122.6 |  Best of 10, k=0.8, 16 tris/leaf
 * |          |         |      |  57.0 | 1.90  |   873,860 | 2.67  |   471,915 | 235,958 |  385  |   37.7   ||  15.2  |  13.6  | 126.5 |  36.0 |  85.0 |  Best of 10, k=0.8,  8 tris/leaf
 * |          |         |      |  68.8 | 2.29  |   881,320 | 2.69  |   670,345 | 335,173 |  491  |   36.0   ||  14.6  |  15.6  | 139.8 |  38.3 |  64.6 |  Best of 10, k=0.8,  4 tris/leaf **
 * |          |         |      |  79.1 | 2.64  |   877,302 | 2.68  |   847,087 | 423,544 |  626  |   34.6   ||  14.5  |  17.0  | 150.5 |  39.2 |  52.2 |  Best of 10, k=0.8,  2 tris/leaf
 * |          |         |      |  84.0 | 2.80  |   880,643 | 2.69  |   927,327 | 463,664 |  586  |   33.8   ||  14.5  |  18.1  | 156.3 |  37.9 |  46.7 |  Best of 10, k=0.8,  1 tris/leaf
 * |          |         |      |       |       |           |       |           |         |       |          ||        |        |       |       |       |
 * |          |         |      | 110.8 | 3.69  | 1,445,775 | 4.41  | 1,044,187 | 522,094 |  676  |   27.5   ||  13.0  |  20.2  | 159.9 |  29.6 |  73.0 |  Best of 10, k=0.00, 4 tris/leaf
 * |          |         |      | 108.0 | 3.60  | 1,405,199 | 4.29  | 1,020,113 | 510,057 |  527  |   26.9   ||  12.7  |  19.7  | 151.8 |  27.8 |  70.1 |  Best of 10, k=0.05, 4 tris/leaf
 * |          |         |      | 107.4 | 3.58  | 1,397,855 | 4.27  | 1,014,775 | 507,388 |  518  |   27.3   ||  12.7  |  19.4  | 148.7 |  28.9 |  70.8 |  Best of 10, k=0.10, 4 tris/leaf
 * |          |         |      | 106.5 | 3.55  | 1,385,452 | 4.23  | 1,007,605 | 503,803 |  604  |   27.1   ||  12.9  |  20.3  | 155.0 |  28.3 |  71.9 |  Best of 10, k=0.15, 4 tris/leaf
 * |          |         |      | 105.6 | 3.52  | 1,371,420 | 4.19  | 1,000,279 | 500,140 |  600  |   27.4   ||  12.7  |  19.4  | 151.2 |  28.4 |  70.4 |  Best of 10, k=0.20, 4 tris/leaf
 * |          |         |      | 103.9 | 3.46  | 1,347,734 | 4.11  |   984,729 | 492,365 |  510  |   27.6   ||  12.6  |  19.6  | 152.0 |  28.4 |  70.3 |  Best of 10, k=0.25, 4 tris/leaf
 * |          |         |      | 102.8 | 3.43  | 1,333,958 | 4.07  |   974,447 | 487,224 |  441  |   28.0   ||  12.7  |  19.5  | 152.8 |  29.6 |  72.1 |  Best of 10, k=0.30, 4 tris/leaf
 * |          |         |      | 101.8 | 3.39  | 1,319,483 | 4.03  |   966,787 | 483,394 |  544  |   27.1   ||  12.1  |  19.5  | 148.7 |  27.3 |  68.0 |  Best of 10, k=0.35, 4 tris/leaf
 * |          |         |      | 100.4 | 3.35  | 1,301,040 | 3.97  |   954,015 | 477,008 |  578  |   27.7   ||  12.3  |  18.5  | 143.7 |  28.9 |  68.9 |  Best of 10, k=0.40, 4 tris/leaf
 * |          |         |      |  99.1 | 3.30  | 1,282,398 | 3.91  |   943,627 | 471,814 |  614  |   28.5   ||  12.4  |  18.6  | 148.3 |  31.0 |  72.5 |  Best of 10, k=0.55, 4 tris/leaf
 * |          |         |      |  94.6 | 3.15  | 1,224,627 | 3.74  |   902,405 | 451,203 |  617  |   27.9   ||  11.7  |  18.5  | 143.6 |  28.4 |  67.7 |  Best of 10, k=0.50, 4 tris/leaf
 * |          |         |      |  89.7 | 2.99  | 1,159,800 | 3.54  |   856,997 | 428,499 |  774  |   29.4   ||  11.9  |  18.1  | 143.6 |  31.1 |  70.7 |  Best of 10, k=0.55, 4 tris/leaf
 * |          |         |      |  87.4 | 2.91  | 1,130,018 | 3.45  |   836,803 | 418,402 |  592  |   29.1   ||  11.6  |  18.1  | 144.7 |  30.2 |  68.1 |  Best of 10, k=0.60, 4 tris/leaf
 * |          |         |      |  84.0 | 2.80  | 1,085,805 | 3.31  |   805,335 | 402,668 |  543  |   29.4   ||  11.4  |  17.7  | 142.1 |  29.8 |  67.2 |  Best of 10, k=0.65, 4 tris/leaf
 * |          |         |      |  79.3 | 2.64  | 1,023,042 | 3.12  |   764,007 | 382,004 |  623  |   30.5   ||  11.0  |  16.3  | 136.8 |  31.1 |  64.8 |  Best of 10, k=0.70, 4 tris/leaf
 * |          |         |      |  74.1 | 2.47  |   953,484 | 2.91  |   717,337 | 358,669 |  597  |   32.1   ||  10.6  |  16.0  | 135.3 |  33.2 |  63.8 |  Best of 10, k=0.75, 4 tris/leaf
 * |          |         |      |  68.5 | 2.28  |   878,411 | 2.68  |   667,837 | 333,919 |  465  |   36.0   ||  10.6  |  15.3  | 136.9 |  38.0 |  64.5 |  Best of 10, k=0.80, 4 tris/leaf
 * |          |         |      |  64.2 | 2.14  |   817,670 | 2.50  |   630,563 | 315,282 |  512  |   42.1   ||  10.6  |  14.9  | 145.5 |  44.9 |  62.7 |  Best of 10, k=0.85, 4 tris/leaf **
 * |          |         |      |  61.6 | 2.05  |   781,461 | 2.38  |   606,957 | 303,479 |  657  |   63.3   ||  11.5  |  14.7  | 170.6 |  70.9 |  69.4 |  Best of 10, k=0.90, 4 tris/leaf
 * |          |         |      |  62.4 | 2.08  |   794,154 | 2.42  |   613,591 | 306,796 |  837  |  124.4   ||  15.8  |  15.8  | 271.3 | 143.9 |  99.0 |  Best of 10, k=0.95, 4 tris/leaf
 * |          |         |      |  68.4 | 2.28  |   867,644 | 2.65  |   672,997 | 336,499 | 1057  |  205.7   ||  18.8  |  19.5  | 350.3 | 232.1 |  92.0 |  Best of 10, k=1.00, 4 tris/leaf
 * |          |         |      |       |       |           |       |           |         |       |          ||        |        |       |       |       |
 * |          |         |      |  64.3 | 2.14  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||  15.3  |  15.0  | 145.4 |  45.1 |  63.4 |  Best of  10, k=0.85, 4 tris/leaf, New baseline
 * |          |         |      |  60.4 | 2.01  |   763,716 | 2.33  |   596,715 | 298,358 |  438  |   36.2   ||  13.5  |  15.0  | 130.5 |  39.1 |  61.5 |  Best of 100
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||  14.0  |  15.0  | 145.4 |  45.1 |  63.4 |  Best of  10, removed cruft
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||  12.7  |  15.0  | 145.4 |  45.1 |  63.4 |  Share empty leaves, NOTE: We're now counting nodes incorrectly
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||  11.7  |  15.0  | 145.4 |  45.1 |  63.4 |  Put nodes in an array
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||  10.8  |  11.3  | 139.2 |  46.6 |  56.3 |  Reduce ray epsilon from 11x plane to 2x (just to see)
 * |          |         |      |       |       |           |       |           |         |       |          ||        |        |       |       |       |
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||  11.8  |  15.0  | 145.4 |  45.1 |  63.4 |  New baseline
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||  13.0  |  20.0  | 154.4 |  43.2 |  74.1 |  Collapse and correct plane checks
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||  13.2  |  17.3  | 149.5 |  44.1 |  68.1 |  Reduce epsilon from 11x to 8x (still correct)
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||  11.1  |  12.0  | 139.9 |  46.3 |  56.8 |  Reduce epsilon 2x (just to see)
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||  17.8  |  16.3  | 233.6 |  28.3 | 184.4 |  Reduce epsilon 0 (just to see) (almost all rays are missing) I think something here is broken.
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||  12.3  |  16.1  | 147.9 |  44.8 |  66.3 |  Back to 8x, Clamp splits with Min/Max
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||  12.4  |  16.1  | 147.9 |  44.8 |  66.3 |  Clamp splits with Lerp
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||  12.4  |  16.1  | 147.9 |  44.8 |  66.3 |  Clamp splits with ternary
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||  12.6  |  16.1  | 147.9 |  44.8 |  66.3 |  Check both sides when ray is parallel
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||  12.5  |  16.1  | 147.9 |  44.8 |  66.3 |  Stupid if restructuring
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||  11.2  |  16.1  | 147.9 |  44.8 |  66.3 |  Remove Asserts
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||  10.3  |  16.1  | 147.9 |  44.8 |  66.3 |  Don't bother with centroid for Triangle_To
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||   8.7  |  16.1  | 147.9 |  44.8 |  66.3 |  Disable....vsync?!?!?!?!
 * |          |         |      |       |       |           |       |           |         |       |          ||        |        |       |       |       |
 * |          |         |      |  45.0 | 1.50  |   820,378 | 2.50  |   631,519 | 315,760 |  406  |   42.2   ||   8.8  |  16.1  | 147.9 |  44.8 |  66.3 |  New baseline
 * |          |         |      |  44.9 | 1.50  |   818,235 | 2.50  |   629,781 | 314,891 |  518  |   43.7   ||   8.8  |  15.4  | 147.8 |  47.5 |  68.5 |  Build back nodes before front nodes (changes the RNG)
 * |          |         |      |  44.9 | 1.50  |   818,235 | 2.50  |   314,891 | 191,009 |  518  |   43.7   ||   8.8  |  15.4  | 147.8 |  47.5 |  68.5 |  Stop counting leaves with nodes, and empty leaves with leaves
 * |          |         |      |  44.9 | 1.50  |   818,235 | 2.50  |   314,891 | 191,009 |  518  |   43.7   ||   9.2  |  15.4  | 132.4 |  47.5 |  68.5 |  Reference nodes by index instead of pointer
 * |          |         |      |       |       |           |       |           |         |       |          ||        |        |       |       |       |
 * | asteroid |         |      |   0.6 | 11.97 |    10,140 | 3.13  |     4,140 |   2,569 |   52  |   17.6   ||   2.5  |   8.8  |  40.9 |  14.3 |  17.1 |  These are very haphazard. I don't fully trust the numbers
 * | station  |         |      |  11.9 |  4.24 |   306,362 | 1.89  |    26,376 |   9,649 |  136  |   30.5   ||   9.5  |  15.8  |  79.1 |  27.3 | 240.1 |
 * |          |         |      |       |       |           |       |           |         |       |          ||        |        |       |       |       |
 * | luffa    |         |      |  44.9 | 1.50  |   818,235 | 2.50  |   314,891 | 191,009 |  518  |   43.7   ||   9.1  |  15.4  | 132.4 |  47.5 |  68.5 |  Use a struct to pass data when building tree nodes.
 * |          |         |      |  45.1 | 1.50  |   821,481 | 2.51  |   315,963 | 191,669 |  578  |   42.1   ||   9.1  |  15.9  | 132.3 |  45.2 |  66.0 |  Don't search for a split plane when we already have few enough triangles (changes the RNG)
 * |          |         |      | 166.7 | 5.56  | 2,929,218 | 8.94  | 1,143,003 | 976,642 |  591  |   80.8   ||   9.0  |  16.3  | 137.9 |  47.8 |  28.0 |  Random bounding box cut when out of valid polygons to split with
 * |          |         |      |  45.1 | 1.50  |   821,481 | 2.51  |   315,963 | 191,669 |  578  |   42.1   ||  10.0  |  15.9  | 132.3 |  45.2 |  66.0 |  Wasn't normalizing n in Triangle_ToPlane
 * |          |         |      |  60.8 | 2.03  | 1,011,646 | 3.09  |   487,869 | 364,063 |  516  |   61.1   ||   9.0  |  16.1  | 136.4 |  49.1 |  27.4 |  Ensure leaves *never* have more than MAX_LEAF_SIZE triangles
 * |          |         |      |  70.3 | 2.34  |   842,267 | 2.57  |   766,160 | 309,795 |  563  |   96.2   ||   8.7  |  16.1  | 139.4 |  52.7 |  26.6 |  Vary k with tree depth 0.85/200 -> 0.00/1000
 * |          |         |      | 103.8 | 3.46  |   846,084 | 2.58  | 1,331,922 | 311,220 |  718  |  194.5   ||   9.3  |  16.3  | 152.9 |  64.2 |  26.1 |  Vary k with tree depth 0.85/400 -> 0.00/1000
 * |          |         |      | 143.1 | 4.77  |   844,388 | 2.58  | 1,986,942 | 310,904 |  721  |  296.8   ||   9.0  |  16.6  | 158.2 |  70.0 |  26.7 |  Vary k with tree depth 0.85/600 -> 0.00/1000
 * |          |         |      |  60.5 | 2.02  |   846,826 | 2.58  |   587,894 | 310,875 |  562  |   68.0   ||   9.1  |  15.8  | 137.1 |  50.0 |  26.3 |  Vary k with tree depth 0.85/100 -> 0.50/1000
 * |          |         |      |  58.3 | 1.94  |   846,096 | 2.58  |   547,737 | 310,575 |  522  |   62.6   ||   8.6  |  15.9  | 135.0 |  50.1 |  26.3 |  Vary k with tree depth 0.85/100 -> 0.25/1000
 * |          |         |      |       |       |           |       |           |         |       |          ||        |        |       |       |       |
 * |          |         |      |  58.5 | 1.95  |   849,718 | 2.59  |   549,291 | 311,727 |  672  |   62.8   ||   9.0  |  16.0  | 135.3 |  50.4 |  26.3 |  Added centroid back to Triangle_ToPlane
 * |          |         |      |  58.5 | 1.95  |   849,718 | 2.59  |   549,291 | 311,727 |  672  |   62.8   ||   8.7  |  16.0  | 135.3 |  50.4 |  26.3 |  Use Triangle_ToPlaneFast in intersection test (lot of variance here, seeing as low as 8.1 us)
 * |          |         |      |  58.5 | 1.95  |   849,719 | 2.59  |   549,291 | 311,727 |  672  |   62.8   ||   8.3  |  16.0  | 135.3 |  50.4 |  26.3 |  Another sample
 * |          |         |      |  58.5 | 1.95  |   849,719 | 2.59  |   549,291 | 311,727 |  672  |   62.8   ||   8.2  |  16.0  | 135.3 |  50.4 |  26.3 |  Store triangles in a single big array
 * |          |         |      |  29.7 | 0.99  |   849,721 | 2.59  |   549,291 | 549,291 |  672  |   62.8   ||   7.8  |  16.0  | 135.3 |  50.4 |  26.3 |  Don't store leaves at all
 * |          |         |      |  29.2 | 0.97  |   841,756 | 2.57  |   309,857 | 309,857 |  582  |   59.8   ||   7.9  |  13.6  | 122.9 |  47.7 |  47.9 |  8 tris/leaf
 * |          |         |      |  38.4 | 1.28  |   841,756 | 2.57  |   309,857 | 309,857 |  582  |   59.8   ||   8.1  |  13.6  | 122.9 |  47.7 |  47.9 |  Fixed BSP size calculation
 * |          |         |      |       |       |           |       |           |         |       |          ||        |        |       |       |       |
 * |          |         |      |  45.9 | 1.53  |   849,721 | 2.59  |   549,291 | 549,291 |  672  |   62.8   ||   8.1  |  16.0  | 135.3 |  50.4 |  26.3 |   4 tris/leaf
 * |          |         |      |  38.4 | 1.28  |   841,756 | 2.57  |   309,857 | 309,857 |  582  |   59.8   ||   8.2  |  13.6  | 122.9 |  47.7 |  47.9 |   8 tris/leaf
 * |          |         |      |  33.9 | 1.13  |   822,172 | 2.51  |   185,560 | 185,560 |  483  |   56.1   ||   8.5  |  12.6  | 115.9 |  46.6 |  85.9 |  16 tris/leaf
 * |          |         |      |  31.3 | 1.04  |   785,255 | 2.40  |   140,767 | 140,767 |  428  |   57.0   ||   9.2  |  11.3  | 101.4 |  43.3 | 146.1 |  32 tris/leaf
 * |          |         |      |       |       |           |       |           |         |       |          ||        |        |       |       |       |
 * |          |         |      |  31.3 | 1.04  |   785,255 | 2.40  |   140,767 | 140,767 |  428  |   57.0   ||   7.4  |  11.3  | 101.4 |  55.9 | 146.0 |  32 tris/leaf, moved profiling code out of loop, rays now always the same (started getting way too much variation), fixed avg depth calculation
 * |          |         |      |  31.3 | 1.04  |   785,255 | 2.40  |   140,767 | 140,767 |  428  |   57.0   ||   6.9  |  11.3  | 101.5 |  55.9 | 146.3 |  Moller's ray-triangle test
 * |          |         |      |  33.9 | 1.13  |   822,172 | 2.51  |   185,560 | 185,560 |  483  |   56.1   ||   6.3  |  12.7  | 116.2 |  62.2 |  86.2 |  16 tris/leaf
 * |          |         |      |  34.4 | 1.15  |   827,393 | 2.53  |   197,842 | 197,842 |  474  |   53.5   ||   6.2  |  12.9  | 115.9 |  60.4 |  78.2 |  14 tris/leaf
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||   6.1  |  13.3  | 118.7 |  62.0 |  67.5 |  12 tris/leaf **
 * |          |         |      |  36.0 | 1.20  |   836,718 | 2.55  |   237,234 | 237,234 |  623  |   57.0   ||   6.1  |  13.7  | 122.5 |  63.5 |  59.9 |  10 tris/leaf
 * |          |         |      |  38.4 | 1.28  |   841,756 | 2.57  |   309,857 | 309,857 |  582  |   59.8   ||   5.9  |  13.7  | 123.2 |  64.4 |  48.1 |   8 tris/leaf
 * |          |         |      |  45.9 | 1.53  |   849,721 | 2.59  |   549,291 | 549,291 |  672  |   62.8   ||   6.0  |  16.1  | 135.6 |  67.4 |  26.5 |   4 tris/leaf
 * |          |         |      |       |       |           |       |           |         |       |          ||        |        |       |       |       |
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||   6.6  |  13.3  | 118.6 |  62.0 |  67.3 |  Barycentric, inline
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||   6.6  |  13.3  | 118.6 |  62.0 |  67.3 |  Barycentric, function
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||   6.2  |  13.3  | 118.7 |  62.0 |  67.5 |  Moller 1, inline
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||   6.2  |  13.3  | 118.7 |  62.0 |  67.5 |  Moller 1, function **
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||   6.4  |  13.3  | 118.6 |  62.0 |  67.3 |  Moller 2, inline
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||   6.4  |  13.3  | 118.6 |  62.0 |  67.3 |  Moller 2, function
 * |          |         |      |       |       |           |       |           |         |       |          ||        |        |       |       |       |
 * |          |         |      |       |       |           |       |           |         |       |          ||        |        |       |       |       |
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||   6.6  |  13.3  | 118.7 |  62.0 |  67.5 | Ray test baseline
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||  43.3  | 267.5  | 369.0 |  46.0 |1009.8 | Sphere test 8x plane epsilon
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||  42.8  | 263.5  | 364.1 |  45.7 | 995.1 | 2x plane epsilon
 * |          |         |      |       |       |           |       |           |         |       |          ||        |        |       |       |       |
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||   7.6  |  13.4  | 117.4 |  61.4 |  66.9 | Ray
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||  44.0  | 268.5  | 370.0 |  45.4 |1016.4 | Sphere r[0.05, 0.30]
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||   8.8  |  13.3  | 116.6 |  60.9 |  66.5 | Ray
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||  10.1  |  44.1  |  97.1 |  35.3 | 168.7 | Sphere r[0.01, 0.10]
 * |          |         |      |       |       |           |       |           |         |       |          ||        |        |       |       |       |
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||  41.4  | 263.5  | 364.1 |  45.7 | 995.1 | Sphere r[0.05, 0.30], 300,000, closest tri
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||  43.8  | 263.5  | 364.1 |  45.7 | 991.9 | Sphere r[0.05, 0.30], 300,000, first tri
 * |          |         |      |  30.2 | 1.01  |   748,050 | 2.28  |   148,262 | 148,262 |  594  |   77.0   ||  36.7  | 175.3  | 270.3 |  51.6 | 930.2 | 32 tris/leaf, 0.25-0.9 split cost
 * |          |         |      |  27.6 | 0.92  |   699,925 | 2.14  |   116,283 | 116,283 |  525  |   79.1   ||  39.8  | 154.1  | 250.3 |  53.6 |1089.3 | 64 tris/leaf, 0.25-0.9 split cost
 * |          |         |      |  27.6 | 0.92  |   699,347 | 2.13  |   116,176 | 116,176 |  471  |   81.5   ||  39.2  | 158.4  | 254.8 |  55.7 |1078.4 | 0.7-0.9 split cost
 * |          |         |      |  28.2 | 0.94  |   730,415 | 2.23  |   102,906 | 102,906 |  526  |   66.5   ||  41.9  | 141.3  | 212.5 |  38.4 |1187.3 | 0.85 split cost
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||   6.7  |  13.3  | 118.7 |  62.0 |  67.5 | Ray, 12 tri/leaf
 * |          |         |      |  31.3 | 1.04  |   785,255 | 2.40  |   140,767 | 140,767 |  428  |   57.0   ||   7.8  |  11.3  | 101.5 |  55.9 | 146.3 | Ray, 32 tri/leaf
 * |          |         |      |  28.2 | 0.94  |   729,757 | 2.23  |   102,917 | 102,917 |  553  |   62.5   ||   9.7  |  10.1  |  86.9 |  51.3 | 261.6 | Ray, 64 tri/leaf
 * |          |         |      |  24.7 | 0.82  |   666,388 | 2.03  |    60,420 |  60,420 |  711  |   81.5   ||  13.3  |   8.5  |  71.0 |  44.9 | 471.0 | Ray, 128 tri/leaf
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||  43.6  | 263.5  | 364.1 |  45.7 | 991.9 | Sphere, 12 tri/leaf
 * |          |         |      |  31.3 | 1.04  |   785,255 | 2.40  |   140,767 | 140,767 |  428  |   57.0   ||  45.1  | 195.8  | 284.8 |  42.4 |1160.1 | Sphere, 32 tri/leaf
 * |          |         |      |  28.2 | 0.94  |   729,757 | 2.23  |   102,917 | 102,917 |  553  |   62.5   ||  42.7  | 146.4  | 219.7 |  39.0 |1206.1 | Sphere, 64 tri/leaf
 * |          |         |      |  24.7 | 0.82  |   666,388 | 2.03  |    60,420 |  60,420 |  711  |   81.5   ||  41.0  |  85.0  | 140.4 |  34.0 |1306.7 | Sphere, 128 tri/leaf
 * |          |         |      |       |       |           |       |           |         |       |          ||        |        |       |       |       |
 * |          |         |      |  50.2 | 1.67  | 1,202,316 | 3.67  |   293,954 | 293,954 |  521  |   61.0   ||   5.6  |  10.5  | 129.7 |  69.6 |  58.0 | Crash fix and reused stack during intersection tests (NEW LAPTOP!!)
 * |          |         |      |  50.2 | 1.67  | 1,202,316 | 3.67  |   293,954 | 293,954 |  521  |   61.0   ||   5.5  |  10.5  | 129.7 |  69.6 |  58.0 | decompose on,  edge split on,   232 oversized leaves, avg  4.9 extra triangles.
 * |          |         |      |  47.0 | 1.57  | 1,129,048 | 3.45  |   269,534 | 269,534 |  451  |   52.6   ||   5.7  |  10.5  | 125.6 |  66.0 |  66.1 | decompose on,  edge split off, 5871 oversized leaves, avg 14.0 extra triangles.
 * |          |         |      |  51.1 | 1.70  | 1,235,277 | 3.77  |   286,193 | 286,193 |  509  |   60.3   ||   5.6  |  10.6  | 131.9 |  70.9 |  57.6 | decompose off, edge split on,  2671 oversized leaves, avg  7.0 extra triangles.
 * |          |         |      |  43.9 | 1.46  | 1,057,694 | 3.23  |   249,205 | 249,205 |  688  |   53.3   ||   6.2  |  10.9  | 133.4 |  70.8 |  81.4 | decompose off, edge split off, 4206 oversized leaves, avg 34.0 extra triangles.
 * |          |         |      |       |       |           |       |           |         |       |          ||        |        |       |       |       |
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||   6.3  |  13.3  | 118.7 |  62.0 |  67.5 | r489, 1e-4, same revision as "Ray, 12 tri/leaf" using new laptop
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||   6.1  |  13.3  | 118.7 |  62.0 |  67.5 | r615, 1e-4, right before epsilon change
 * |          |         |      |  38.3 | 1.28  |   908,240 | 2.77  |   233,512 | 233,512 |  495  |   54.4   ||   6.0  |  11.9  | 120.8 |  63.9 |  62.4 | r616, 6e-5, epsilon change
 * |          |         |      |  35.0 | 1.17  |   831,193 | 2.54  |   212,601 | 212,601 |  649  |   54.9   ||   6.2  |  13.3  | 118.7 |  62.0 |  67.5 | r616, 1e-4, confirmation that epsilon is the only contributor
 * |          |         |      |  40.3 | 1.34  |   964,653 | 2.94  |   234,129 | 234,129 |  542  |   61.3   ||   6.1  |  13.1  | 118.8 |  62.8 |  67.6 | r621, 1e-4, crash fix with original epsilon
 * |          |         |      |  50.2 | 1.67  | 1,202,316 | 3.67  |   293,954 | 293,954 |  521  |   61.0   ||   6.0  |  10.5  | 129.7 |  69.6 |  58.0 | r621, 1e-5, crash fix with reduced epsilon
 * |          |         |      |  40.3 | 1.34  |   964,653 | 2.94  |   234,129 | 234,129 |  542  |   61.3   ||   5.7  |  13.1  | 118.8 |  62.8 |  67.6 | r630, 1e-4, performance confirmation on latest revision **
 * |          |         |      |  50.2 | 1.67  | 1,202,316 | 3.67  |   293,954 | 293,954 |  521  |   61.0   ||   5.6  |  10.5  | 129.7 |  69.6 |  58.0 | r630, 1e-5, performance confirmation on latest revision
 */

#[derive(Clone)]
#[repr(C)]
pub struct Bsp {
    pub root_node: BspNodeRef,
    pub empty_leaf: BspNodeRef,
    pub nodes: Vec<BspNode>,
    pub triangles: Vec<Triangle>,
    shader: Shader,
    // BSP_PROFILE (
    //     BSPDebug_Data profilingData;
    // )
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct BspNode {
    pub plane: Plane,
    pub child: [BspNodeRef; 2],
}

#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
pub struct BspNodeRef {
    pub index: i32,
    pub triangle_count: u8,
}

#[luajit_ffi_gen::luajit_ffi(
    name = "BSPNodeRef",
    clone = true,
    typedef = "
        int32 index;
        uint8 triangleCount;"
)]
impl BspNodeRef {}

#[derive(Clone)]
#[repr(C)]
pub struct IntersectSphereProfiling {
    pub nodes: i32,
    pub leaves: i32,
    pub triangles: i32,
    pub triangle_tests: Vec<TriangleTest>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TriangleTest {
    pub triangle: Triangle,
    pub hit: bool,
}

#[luajit_ffi_gen::luajit_ffi(name = "BSPNodeRel")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BspNodeRel {
    Parent = 0,
    Back = 1,
    Front = 2,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DelayRay {
    pub node_ref: BspNodeRef,
    pub t_min: f32,
    pub t_max: f32,
    pub depth: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Delay {
    pub node_ref: BspNodeRef,
    pub depth: i32,
}

const ROOT_NODE_INDEX: i32 = 1;
const EMPTY_LEAF_INDEX: i32 = 1;

#[luajit_ffi_gen::luajit_ffi(name = "BSP")]
impl Bsp {
    #[bind(name = "Create")]
    pub fn new(mesh: &Mesh) -> Self {
        // Assert(LEAF_TRIANGLE_COUNT <= MAX_LEAF_TRIANGLE_COUNT);

        /* NOTE: This function will use memory proportional to 2x the mesh memory.
         *        There will be one copy of all the polygons & vertices in the initial
         *        list of polygons passed to BSPBuild_CreateNode, which will then create new
         *        lists of polygons for the back and front, but will reuse the vertices
         *        from the original list. Therefore we never have more vertices than we
         *        actually need during tree building (aside from the fact that polygons
         *        don't share vertices), but we do have 2 copys of each polygon
         *        temporarily. Then during BSPBuild_OptimizeTree, all the resulting polygons
         *        will be decomposed into triangles which will temporarily store 2
         *        copies of all vertices. The 2x figure is slightly hand-wavy because
         *        splitting will increase the total number of vertices, but I'm
         *        assuming that doesn't get too out of hand for now. Since the mesh
         *        stores indices and vertex attributes I expect the proportionality
         *        constant to be in the ballpark of 0.5 */

        let index_data = mesh.get_index_data();
        let vertex_data = mesh.get_vertex_data();

        /* TODO : Implement some form of soft abort when the incoming mesh is bad. */
        // CHECK2 (
        //     if (Mesh_Validate(mesh) != Error_None) return 0;
        // )

        let mut node_data = BspBuildNodeData {
            polygons: Vec::new(),
            valid_polygon_count: 0,
            triangle_count: 0,
            depth: 0,
        };
        node_data.triangle_count = index_data.len() / 3;
        node_data.valid_polygon_count = index_data.len() / 3;

        node_data.polygons.reserve(node_data.triangle_count);
        for i in (0..index_data.len()).step_by(3) {
            let i0 = index_data[i];
            let i1 = index_data[i + 1];
            let i2 = index_data[i + 2];
            let v0 = vertex_data[i0 as usize].p;
            let v1 = vertex_data[i1 as usize].p;
            let v2 = vertex_data[i2 as usize].p;

            node_data.polygons.push(PolygonEx {
                inner: Polygon {
                    vertices: vec![v0, v1, v2],
                },
                flags: 0,
            });
        }

        /* Build */
        let mut rng = Rng::new(1235);
        let mut bsp_build = BspBuild {
            node_count: 0,
            leaf_count: 0,
            triangle_count: 0,
        };
        let build_root_node = bsp_build.create_node(&mut node_data, &mut rng);

        /* Optimize */
        let null_leaf = Triangle {
            vertices: [Vec3::ZERO; 3],
        };

        let mut triangles = Vec::with_capacity(bsp_build.triangle_count + 2);
        triangles.push(null_leaf);
        triangles.push(null_leaf);

        let empty_leaf = BspNodeRef {
            index: -EMPTY_LEAF_INDEX,
            triangle_count: 0,
        };

        let null_node = BspNode {
            plane: Plane {
                n: Vec3::ZERO,
                d: 0.,
            },
            child: [BspNodeRef {
                index: 0,
                triangle_count: 0,
            }; 2],
        };

        let mut nodes = Vec::with_capacity(bsp_build.node_count + 1);
        nodes.push(null_node);

        let mut bsp = Self {
            root_node: Default::default(),
            empty_leaf,
            nodes,
            triangles,
            shader: Shader::load("vertex/wvp", "fragment/simple_color"),
        };

        bsp.root_node = bsp.optimize_tree(&build_root_node);

        // #if BSP_PROFILE && CHECK_LEVEL >= 2
        //     self->profilingData.oversizedNodes = bspBuild.oversizedNodes;
        //     self->profilingData.avgOversizeAmount = bspBuild.avgOversizeAmount;
        //     if (bspBuild.oversizedNodes > 0) {
        //     Warn("BSP_Create: Created %i oversized leaves with an average excess of %.1f triangles.", bspBuild.oversizedNodes, bspBuild.avgOversizeAmount);
        //     }
        // #endif

        // Assert(ArrayList_GetSize(self->nodes)     == ArrayList_GetCapacity(self->nodes));
        // Assert(ArrayList_GetSize(self->triangles) == ArrayList_GetCapacity(self->triangles));
        // BSP_PROFILE(BSPBuild_AnalyzeTree(self, mesh, self->rootNode, 0);)

        bsp
    }

    pub fn intersect_ray(&self, ray: &mut Ray, t_hit: &mut f32) -> bool {
        // Assert(RAY_INTERSECTION_EPSILON > PLANE_THICKNESS_EPSILON);

        *t_hit = f32::MAX;

        let mut node_ref = self.root_node;
        let t_epsilon = (8.0 * 1e-4 / ray.dir.length() as f64) as f32;
        let mut hit = false;
        let mut depth = 0;
        let mut max_depth = 0;

        let mut ray_stack = vec![];

        loop {
            max_depth = i32::max(depth, max_depth);

            if node_ref.index >= 0 {
                let node = &self.nodes[node_ref.index as usize];
                //BSP_PROFILE(self->profilingData.ray.nodes++;)

                let dist = Vec3::dot(node.plane.n, ray.p.as_vec3()) - node.plane.d;
                let denom = -Vec3::dot(node.plane.n, ray.dir.as_vec3());

                /* Near means the side of the plane the point p is on. */
                /* Early means the side of the plane we'll check first. */
                let near_index = (dist > 0.0) as i32;
                let mut early_index = near_index;

                if denom != 0.0 {
                    /* Ray not parallel to plane */
                    let t = dist / denom;
                    let plane_begin = t - t_epsilon;
                    let plane_end = t + t_epsilon;

                    if plane_begin as f64 >= ray.t_max {
                        /* Entire ray lies on the near side */
                    } else if plane_end as f64 <= ray.t_min {
                        /* Entire ray lies on one side */
                        early_index = (t >= 0.0) as i32 ^ near_index;
                    } else {
                        /* Ray touches thick plane */
                        early_index = (t < 0.0) as i32 ^ near_index;

                        /* Don't let the ray 'creep past' tMin/tMax */
                        let min = f32::max(plane_begin, ray.t_min as f32);
                        let max = f32::min(plane_end, ray.t_max as f32);

                        let d = DelayRay {
                            node_ref: node.child[(1 ^ early_index) as usize],
                            t_min: min,
                            t_max: ray.t_max as f32,
                            depth,
                        };
                        ray_stack.push(d);

                        ray.t_max = max as f64;
                    }
                } else {
                    /* Ray parallel to plane. */
                    if f64::abs(dist as f64) < 8.0f64 * 1e-4f64 {
                        early_index = near_index;

                        let d = DelayRay {
                            node_ref: node.child[(1 ^ early_index) as usize],
                            t_min: ray.t_min as f32,
                            t_max: ray.t_max as f32,
                            depth,
                        };
                        ray_stack.push(d);
                    } else {
                        /* Ray outside of thick plane */
                    }
                }

                depth += 1;
                node_ref = node.child[early_index as usize];
            } else {
                let leaf_index = -node_ref.index;
                // BSP_PROFILE(self->profilingData.ray.leaves++;)

                let mut i = 0u8;
                while (i as i32) < node_ref.triangle_count as i32 {
                    let triangle = &self.triangles[leaf_index as usize + i as usize];
                    // BSP_PROFILE(self->profilingData.ray.triangles++;)

                    let mut t = 0.;
                    // if (Intersect::ray_triangle_barycentric(ray, triangle, tEpsilon, &t)) {
                    if Intersect::ray_triangle_moller1(ray, triangle, &mut t) {
                        // if (Intersect::ray_triangle_moller2(ray, triangle, &t)) {
                        // if (Intersect::ray_triangle_badouel(ray, triangle, tEpsilon, &t)) {
                        if !hit || t < *t_hit {
                            hit = true;
                            *t_hit = t;
                        }
                    }
                    i = i.wrapping_add(1);
                }

                if hit {
                    break;
                }

                if let Some(d) = ray_stack.pop() {
                    node_ref = d.node_ref;
                    ray.t_min = d.t_min as f64;
                    ray.t_max = d.t_max as f64;
                    depth = d.depth;
                } else {
                    break;
                }
            }
        }

        // BSP_PROFILE (
        //     self->profilingData.ray.count++;
        //     self->profilingData.ray.depth += maxDepth;
        // )

        hit
    }

    pub fn intersect_line_segment(&self, line_segment: &LineSegment, p_hit: &mut Vec3) -> bool {
        let mut ray = Ray {
            p: line_segment.p0,
            dir: line_segment.p1.as_dvec3() - line_segment.p0.as_dvec3(),
            t_min: 0.0,
            t_max: 1.0,
        };
        let mut t = 0.;
        if self.intersect_ray(&mut ray, &mut t) {
            *p_hit = ray.get_point(t as f64).as_vec3();
            true
        } else {
            false
        }
    }

    pub fn intersect_sphere(&self, sphere: &Sphere, p_hit: &mut Vec3) -> bool {
        // Assert(SPHERE_INTERSECTION_EPSILON > PLANE_THICKNESS_EPSILON);

        let mut node_ref = self.root_node;
        let mut hit = false;
        let mut depth = 0;
        let mut max_depth = 0;

        let mut node_stack = vec![];

        loop {
            max_depth = i32::max(depth, max_depth);

            if node_ref.index >= 0 {
                let node = &self.nodes[node_ref.index as usize];
                // BSP_PROFILE(self->profilingData.sphere.nodes++;)

                let dist = Vec3::dot(node.plane.n, sphere.p) - node.plane.d;
                if dist > sphere.r + 2.0 * 1e-4 {
                    /* Entirely in front half-space */
                    node_ref = node.child[FRONT_INDEX];
                } else if dist < -(sphere.r + 2.0 * 1e-4) {
                    /* Entirely in back half-space */
                    node_ref = node.child[BACK_INDEX];
                } else {
                    /* Straddling the thick plane */
                    let d = Delay {
                        node_ref: node.child[BACK_INDEX],
                        depth,
                    };
                    node_stack.push(d);
                    node_ref = node.child[FRONT_INDEX];
                }

                depth += 1;
            } else {
                let leaf_index = -node_ref.index;
                // BSP_PROFILE(self->profilingData.sphere.leaves++;)

                let mut i = 0u8;
                while i < node_ref.triangle_count {
                    let triangle = &self.triangles[leaf_index as usize + i as usize];
                    // BSP_PROFILE(self->profilingData.sphere.triangles++;)

                    let mut p_hit2 = Vec3::ZERO;
                    if Intersect::sphere_triangle(sphere, triangle, &mut p_hit2) {
                        hit = true;
                        *p_hit = p_hit2;
                        break;
                    }
                    i = i.wrapping_add(1);
                }

                if hit {
                    break;
                }

                if let Some(d) = node_stack.pop() {
                    node_ref = d.node_ref;
                    depth = d.depth;
                } else {
                    break;
                }
            }
        }

        // BSP_PROFILE (
        //     self->profilingData.sphere.count++;
        //     self->profilingData.sphere.depth += maxDepth;
        // )

        hit
    }

    /*
    BSP_PROFILE (
    static void analyze_tree(&self, Mesh* mesh, BSPNodeRef nodeRef, int32 depth) {
      BSPDebug_Data* pd = &self->profilingData;

      /* TODO : Do this while building */

      /* All */
      pd->maxDepth = Max(pd->maxDepth, depth);

      /* Internal */
      if (nodeRef.index >= 0) {
        BSPNode* node = ArrayList_GetPtr(self->nodes, nodeRef.index);
        BSPBuild_AnalyzeTree(self, mesh, node->child[BackIndex] , depth + 1);
        BSPBuild_AnalyzeTree(self, mesh, node->child[FrontIndex], depth + 1);
      }
      /* Leaf */
      else {
        pd->leafCount++;
        pd->avgLeafDepth = Lerp(pd->avgLeafDepth, (float) depth, 1.0f / (float) pd->leafCount);
      }

      /* Root */
      if (depth == 0) {
        const float BToMiB = 1.0f / 1024.0f / 1024.0f;

        pd->nodeCount = ArrayList_GetCapacity(self->nodes);
        pd->usedMiB += pd->nodeCount * sizeof(BSPNode);

        pd->triCount += ArrayList_GetCapacity(self->triangles);
        pd->usedMiB += pd->triCount * sizeof(Triangle);

        pd->meshTriCount = Mesh_GetIndexCount(mesh) / 3;
        pd->meshMiB = (float) Mesh_GetIndexCount(mesh) * sizeof(int32);
        pd->meshMiB = (float) Mesh_GetVertexCount(mesh) * sizeof(Vertex);
        pd->meshMiB *= BToMiB;

        pd->usedMiB += sizeof(BSP);
        pd->usedMiB *= BToMiB;

        pd->triCountRatio = (float) pd->triCount / (float) pd->meshTriCount;
        pd->usedMiBRatio = (float) pd->usedMiB / (float) pd->meshMiB;
      }
    }
    */

    pub fn get_node(&self, node_ref: BspNodeRef, relationship: BspNodeRel) -> BspNodeRef {
        if node_ref.index == 0 {
            return self.root_node;
        }

        let mut node = None;
        if node_ref.index > 0 {
            node = Some(&self.nodes[node_ref.index as usize]);
        }

        let mut new_node = BspNodeRef {
            index: 0,
            triangle_count: 0,
        };
        if relationship == BspNodeRel::Parent {
            if node_ref.index != 0 {
                for i in 0..self.nodes.len() {
                    let node_to_check = &self.nodes[i];
                    if (*node_to_check).child[BACK_INDEX].index == node_ref.index
                        || (*node_to_check).child[FRONT_INDEX].index == node_ref.index
                    {
                        new_node.index = i as i32;
                        break;
                    }
                }
            }
        } else if relationship == BspNodeRel::Back {
            if let Some(node) = &node {
                new_node = node.child[BACK_INDEX];
            }
        } else if relationship == BspNodeRel::Front {
            if let Some(node) = &node {
                new_node = node.child[FRONT_INDEX];
            }
        } else {
            panic!("BSPDebug_GetNode: Unhandled case: {}", relationship as i32,)
        }

        if new_node.index != 0 {
            new_node
        } else {
            node_ref
        }
    }

    pub fn draw_node(&mut self, node_ref: BspNodeRef, color: &Color) {
        // Assert(nodeRef.index);

        if node_ref.index > 0 {
            self.draw_node(self.nodes[node_ref.index as usize].child[BACK_INDEX], color);
            self.draw_node(
                self.nodes[node_ref.index as usize].child[FRONT_INDEX],
                color,
            );
        } else {
            self.shader.start();
            self.shader
                .set_float4("color", color.r, color.g, color.b, color.a);
            let leaf_index = -node_ref.index;
            for i in 0..node_ref.triangle_count {
                let triangle = &self.triangles[leaf_index as usize + i as usize];
                Draw::tri3(
                    &triangle.vertices[0],
                    &triangle.vertices[1],
                    &triangle.vertices[2],
                );
            }
            self.shader.stop();
        };
    }

    pub fn draw_node_split(&mut self, node_ref: BspNodeRef) {
        // Assert(nodeRef.index);

        RenderState::push_blend_mode(BlendMode::Alpha);
        RenderState::push_cull_face(CullFace::Back);
        RenderState::push_depth_test(true);
        RenderState::push_wireframe(true);

        if node_ref.index > 0 {
            let child = self.nodes[node_ref.index as usize].child;
            self.draw_node(child[BACK_INDEX], &Color::new(0.5, 0.3, 0.3, 0.4));
            self.draw_node(child[FRONT_INDEX], &Color::new(0.3, 0.5, 0.3, 0.4));

            let node = &self.nodes[node_ref.index as usize];

            /* Plane */
            let origin = Vec3::new(0., 0., 0.);
            let t = Vec3::dot(node.plane.n, origin) - node.plane.d;
            let closest_point = origin - (node.plane.n * t);
            RenderState::push_wireframe(false);
            self.shader.start();
            self.shader.set_float4("color", 0.3, 0.5, 0.3, 0.4);
            Draw::plane(&closest_point, &node.plane.n, 2.0);
            self.shader.set_float4("color", 0.5, 0.3, 0.3, 0.4);
            let neg: Vec3 = node.plane.n * -1.0;
            Draw::plane(&closest_point, &neg, 2.0);
            self.shader.stop();
            RenderState::pop_wireframe();
        } else {
            /* Leaf */
            self.draw_node(node_ref, &Color::new(0.5, 0.5, 0.3, 0.4));
        }

        RenderState::pop_wireframe();
        RenderState::pop_depth_test();
        RenderState::pop_cull_face();
        RenderState::pop_blend_mode();

        self.shader.stop(); // TODO: no start?
    }

    pub fn draw_line_segment(&mut self, line_segment: &LineSegment, eye: &Position) {
        let mut p_hit = Vec3::ZERO;

        self.shader.start();
        if self.intersect_line_segment(line_segment, &mut p_hit) {
            self.shader.set_float4("color", 0.0, 1.0, 0.0, 0.1);
            Draw::line3(
                &(*line_segment).p0.relative_to(*eye),
                &Position::from_vec(p_hit).relative_to(*eye),
            );

            self.shader.set_float4("color", 1.0, 0.0, 0.0, 1.0);
            Draw::line3(
                &Position::from_vec(p_hit).relative_to(*eye),
                &(*line_segment).p1.relative_to(*eye),
            );

            Draw::point_size(5.0);
            Draw::point3(p_hit.x, p_hit.y, p_hit.z);
        } else {
            self.shader.set_float4("color", 0.0, 1.0, 0.0, 1.0);
            Draw::line3(
                &(*line_segment).p0.relative_to(*eye),
                &(*line_segment).p1.relative_to(*eye),
            );
        };
        self.shader.stop();
    }

    pub fn draw_sphere(&mut self, sphere: &Sphere) {
        let mut p_hit = Vec3::ZERO;

        self.shader.start();
        if self.intersect_sphere(sphere, &mut p_hit) {
            RenderState::push_wireframe(false);
            self.shader.set_float4("color", 1.0, 0.0, 0.0, 0.3);
            Draw::sphere(&sphere.p, sphere.r);
            RenderState::pop_wireframe();

            self.shader.set_float4("color", 1.0, 0.0, 0.0, 1.0);
            Draw::sphere(&sphere.p, sphere.r);

            RenderState::push_depth_test(false);
            Draw::point_size(8.0);
            Draw::point3(p_hit.x, p_hit.y, p_hit.z);
            RenderState::pop_depth_test();
        } else {
            RenderState::push_wireframe(false);
            self.shader.set_float4("color", 0.0, 1.0, 0.0, 0.3);
            Draw::sphere(&sphere.p, sphere.r);
            RenderState::pop_wireframe();

            self.shader.set_float4("color", 0.0, 1.0, 0.0, 1.0);
            Draw::sphere(&sphere.p, sphere.r);
        };
        self.shader.stop();
    }

    // static void print_profiling_data(&self, BSPDebug_IntersectionData* data, double totalTime) {
    //     #if ENABLE_BSP_PROFILING
    //       BSPDebug_Data* pd = &self->profilingData;

    //       float us = (float) (totalTime * 1000.0 * 1000.0);
    //       float avgus     = (float) us              / data->count;
    //       float avgLeaves = (float) data->leaves    / data->count;
    //       float avgNodes  = (float) data->nodes     / data->count;
    //       float avgTris   = (float) data->triangles / data->count;
    //       float avgDepth  = (float) data->depth     / data->count;

    //       char buffer[256];
    //       /*                                            name       tris      mb    bsp mb   mbr    tris   trir     nod   lv    maxd      lvd        ray us    rayl     rayn    rayd    rayt */
    //       snprintf(buffer, (size_t) Array_GetSize(buffer), "* |          |         |      | %5.1f | %4.2f  | %9d | %4.2f  | %9d | %7d |  %3d  |  %5.1f   ||  %4.1f  |  %4.1f  | %5.1f | %5.1f | %5.1f |\n",
    //         pd->usedMiB, pd->usedMiBRatio, pd->triCount, pd->triCountRatio,
    //         pd->nodeCount, pd->leafCount, pd->maxDepth, pd->avgLeafDepth,
    //         avgus, avgLeaves, avgNodes, avgDepth, avgTris
    //       );
    //       puts(buffer);
    //     #else
    //       Warn("BSP_PrintProfilingData: BSP profiling is not enabled. Set ENABLE_BSP_PROFILING to enable this function.");
    //       UNUSED(self); UNUSED(data); UNUSED(totalTime);
    //       UNUSED(&BSPDebug_PrintProfilingData);
    //     #endif
    //   }

    pub fn print_ray_profiling_data(&self, _total_time: f64) {
        // #if ENABLE_BSP_PROFILING
        //   BSPDebug_PrintProfilingData(self, &self->profilingData.ray, totalTime);
        // #else
        warn!(
            "BSP_PrintRayProfilingData: BSP profiling is not enabled. Set ENABLE_BSP_PROFILING to enable this function."
        );
    }

    pub fn print_sphere_profiling_data(&self, _total_time: f64) {
        // #if ENABLE_BSP_PROFILING
        //     BSPDebug_PrintProfilingData(self, &self->profilingData.sphere, totalTime);
        // #else
        warn!(
            "BSP_PrintSphereProfilingData: BSP profiling is not enabled. Set ENABLE_BSP_PROFILING to enable this function."
        );
    }

    pub fn get_intersect_sphere_triangles(
        &self,
        sphere: &Sphere,
        sphere_prof: &mut IntersectSphereProfiling,
    ) -> bool {
        // Assert(SPHERE_INTERSECTION_EPSILON > PLANE_THICKNESS_EPSILON);

        let mut node_ref = self.root_node;
        let mut hit = false;
        let mut depth = 0;
        let mut max_depth = 0;

        let mut node_stack = vec![];

        loop {
            max_depth = i32::max(depth, max_depth);

            if node_ref.index >= 0 {
                let node = &self.nodes[node_ref.index as usize];
                sphere_prof.nodes += 1;

                let dist = Vec3::dot(node.plane.n, sphere.p) - node.plane.d;
                if dist > sphere.r + 2.0 * 1e-4 {
                    /* Entirely in front half-space */
                    node_ref = node.child[FRONT_INDEX];
                } else if (dist) < -(sphere.r + 2.0 * 1e-4) {
                    /* Entirely in back half-space */
                    node_ref = node.child[BACK_INDEX];
                } else {
                    /* Straddling the thick plane */
                    let d = Delay {
                        node_ref: node.child[BACK_INDEX],
                        depth,
                    };
                    node_stack.push(d);
                    node_ref = node.child[FRONT_INDEX];
                }

                depth += 1;
            } else {
                let leaf_index = -node_ref.index;
                sphere_prof.leaves += 1;

                for i in 0..node_ref.triangle_count {
                    let triangle = &self.triangles[leaf_index as usize + i as usize];
                    sphere_prof.triangles += 1;

                    let mut p_hit2 = Vec3::ZERO;
                    if Intersect::sphere_triangle(sphere, triangle, &mut p_hit2) {
                        let t = TriangleTest {
                            triangle: *triangle,
                            hit: true,
                        };
                        sphere_prof.triangle_tests.push(t);
                        hit = true;
                        break;
                    }

                    let t = TriangleTest {
                        triangle: *triangle,
                        hit: false,
                    };
                    sphere_prof.triangle_tests.push(t);
                }

                if hit {
                    break;
                }

                if let Some(d) = node_stack.pop() {
                    node_ref = d.node_ref;
                    depth = d.depth;
                } else {
                    break;
                }
            }
        }

        hit
    }

    pub fn get_leaf(&self, leaf_index: i32) -> BspNodeRef {
        let mut index: i32 = -1;
        for node in &self.nodes {
            if node.child[0].index < 0 {
                let prev_index = index;
                index += 1;
                if prev_index == leaf_index {
                    return node.child[0];
                }
            }
            if node.child[1].index < 0 {
                let prev_index = index;
                index += 1;
                if prev_index == leaf_index {
                    return node.child[1];
                }
            }
        }
        BspNodeRef {
            index: ROOT_NODE_INDEX,
            triangle_count: 0 as u8,
        }
    }
}

impl Bsp {
    fn optimize_tree(&mut self, build_node: &BspBuildNode) -> BspNodeRef {
        if build_node.child[BACK_INDEX].is_some() || build_node.child[FRONT_INDEX].is_some() {
            /* Node */
            // Assert(ArrayList_GetSize(self->nodes) < ArrayList_GetCapacity(self->nodes));

            let dummy = BspNode {
                plane: build_node.plane,
                child: [BspNodeRef {
                    index: 0,
                    triangle_count: 0,
                }; 2],
            };
            let node_index = self.nodes.len() as i32;
            self.nodes.push(dummy);

            let mut node_child = [BspNodeRef {
                index: 0,
                triangle_count: 0,
            }; 2];
            if let Some(child) = &build_node.child[BACK_INDEX] {
                node_child[BACK_INDEX] = self.optimize_tree(child);
            }
            if let Some(child) = &build_node.child[FRONT_INDEX] {
                node_child[FRONT_INDEX] = self.optimize_tree(child);
            }
            let node = &mut self.nodes[node_index as usize];
            if build_node.child[BACK_INDEX].is_some() {
                node.child[BACK_INDEX] = node_child[BACK_INDEX];
            }
            if build_node.child[FRONT_INDEX].is_some() {
                node.child[FRONT_INDEX] = node_child[FRONT_INDEX];
            }

            BspNodeRef {
                index: node_index,
                triangle_count: 0 as u8,
            }
        } else {
            /* Leaf */
            if build_node.polygons.is_empty() {
                return self.empty_leaf;
            }

            let leaf_index = self.triangles.len();

            for polygon in &build_node.polygons {
                // Assert(
                //     ArrayList_GetSize(self->triangles) +
                //     ArrayList_GetSize(polygon->vertices) - 2
                //     <= ArrayList_GetCapacity(self->triangles)
                // );
                let mut triangles = polygon.inner.convex_to_triangles();
                self.triangles.append(&mut triangles);
            }

            let leaf_len = (self.triangles.len() - leaf_index) as u8;
            BspNodeRef {
                index: -(leaf_index as i32),
                triangle_count: leaf_len,
            }
        }
    }
}
