use internal::*;

use super::*;
use crate::logging::warn;
use crate::math::*;

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
pub struct BSP {
    pub rootNode: BSPNodeRef,
    pub emptyLeaf: BSPNodeRef,
    pub nodes: Vec<BSPNode>,
    pub triangles: Vec<Triangle>,
    // BSP_PROFILE (
    //     BSPDebug_Data profilingData;
    // )
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct BSPNode {
    pub plane: Plane,
    pub child: [BSPNodeRef; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct BSPNodeRef {
    pub index: i32,
    pub triangleCount: u8,
}

#[derive(Clone)]
#[repr(C)]
pub struct IntersectSphereProfiling {
    pub nodes: i32,
    pub leaves: i32,
    pub triangles: i32,
    pub triangleTests: Vec<TriangleTest>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TriangleTest {
    pub triangle: *mut Triangle,
    pub hit: bool,
}

pub type BSPNodeRel = u8;

#[repr(C)]
pub struct BSPBuild {
    pub rootNode: *mut BSPBuildNode,
    pub rng: Box<Rng>,
    pub nodeCount: i32,
    pub leafCount: i32,
    pub triangleCount: i32,
    // CHECK2 (
    //     int32 nextNodeID;
    //     int32 oversizedNodes;
    //     float avgOversizeAmount;
    // )
}

#[derive(Clone)]
#[repr(C)]
pub struct BSPBuildNode {
    pub plane: Plane,
    pub child: [*mut BSPBuildNode; 2],
    pub polygons: Vec<PolygonEx>,
    // CHECK2 (
    //     int32 id;
    //     BSPBuild_Node* parent;
    //     Vec3f planeCenter;
    // )
}

#[derive(Clone)]
#[repr(C)]
pub struct PolygonEx {
    pub inner: Polygon,
    pub flags: PolygonFlag,
}
pub type PolygonFlag = u8;

#[derive(Clone)]
#[repr(C)]
pub struct BSPBuildNodeData {
    pub polygons: Vec<PolygonEx>,
    pub validPolygonCount: i32,
    pub triangleCount: i32,
    pub depth: u16,
    //Box3f boundingBox;
    //uint8 cutIndex;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DelayRay {
    pub nodeRef: BSPNodeRef,
    pub tMin: f32,
    pub tMax: f32,
    pub depth: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Delay {
    pub nodeRef: BSPNodeRef,
    pub depth: i32,
}

#[no_mangle]
pub static BSPNodeRel_Parent: BSPNodeRel = 0 as BSPNodeRel;

#[no_mangle]
pub static BSPNodeRel_Back: BSPNodeRel = 1 as BSPNodeRel;

#[no_mangle]
pub static BSPNodeRel_Front: BSPNodeRel = 2 as BSPNodeRel;

const BackIndex: i32 = 0;
const FrontIndex: i32 = 1;
static mut RootNodeIndex: i32 = 1;
static mut EmptyLeafIndex: i32 = 1;

pub static mut rayStack: Vec<DelayRay> = Vec::new();

#[no_mangle]
pub unsafe extern "C" fn BSP_IntersectRay(
    this: &mut BSP,
    rayPtr: *const Ray,
    tHit: *mut f32,
) -> bool {
    // Assert(RAY_INTERSECTION_EPSILON > PLANE_THICKNESS_EPSILON);

    let mut ray: Ray = *rayPtr;
    *tHit = f32::MAX;

    let mut nodeRef: BSPNodeRef = this.rootNode;
    let tEpsilon: f32 = (8.0f64 * 1e-4f64 / ray.dir.length() as f64) as f32;
    let mut hit: bool = false;
    let mut depth: i32 = 0;
    let mut maxDepth: i32 = 0;

    loop {
        maxDepth = i32::max(depth, maxDepth);

        if nodeRef.index >= 0 {
            let node: &mut BSPNode = &mut this.nodes[nodeRef.index as usize];
            //BSP_PROFILE(self->profilingData.ray.nodes++;)

            let dist: f32 = Vec3::dot((*node).plane.n, ray.p.as_vec3()) - (*node).plane.d;
            let denom: f32 = -Vec3::dot((*node).plane.n, ray.dir.as_vec3());

            /* Near means the side of the plane the point p is on. */
            /* Early means the side of the plane we'll check first. */
            let nearIndex: i32 = (dist > 0.0f32) as i32;
            let mut earlyIndex: i32 = nearIndex;

            if denom != 0.0f32 {
                /* Ray not parallel to plane */
                let t: f32 = dist / denom;
                let planeBegin: f32 = t - tEpsilon;
                let planeEnd: f32 = t + tEpsilon;

                if planeBegin >= ray.tMax as f32 {
                    /* Entire ray lies on the near side */
                } else if planeEnd <= ray.tMin as f32 {
                    /* Entire ray lies on one side */
                    earlyIndex = (t >= 0.0f32) as i32 ^ nearIndex;
                } else {
                    /* Ray touches thick plane */
                    earlyIndex = (t < 0.0f32) as i32 ^ nearIndex;

                    /* Don't let the ray 'creep past' tMin/tMax */
                    let min: f32 = f32::max(planeBegin, ray.tMin as f32);
                    let max: f32 = f32::min(planeEnd, ray.tMax as f32);

                    let d: DelayRay = DelayRay {
                        nodeRef: (*node).child[(1 ^ earlyIndex) as usize],
                        tMin: min,
                        tMax: ray.tMax as f32,
                        depth,
                    };
                    rayStack.push(d);

                    ray.tMax = max as f64;
                }
            } else {
                /* Ray parallel to plane. */
                if f64::abs(dist as f64) < 8.0f64 * 1e-4f64 {
                    earlyIndex = nearIndex;

                    let d: DelayRay = DelayRay {
                        nodeRef: (*node).child[(1 ^ earlyIndex) as usize],
                        tMin: ray.tMin as f32,
                        tMax: ray.tMax as f32,
                        depth,
                    };
                    rayStack.push(d);
                } else {
                    /* Ray outside of thick plane */
                }
            }

            depth += 1;
            nodeRef = (*node).child[earlyIndex as usize];
        } else {
            let leafIndex = -nodeRef.index;
            // BSP_PROFILE(self->profilingData.ray.leaves++;)

            let mut i: u8 = 0 as u8;
            while (i as i32) < nodeRef.triangleCount as i32 {
                let triangle = &this.triangles[leafIndex as usize + i as usize];
                // BSP_PROFILE(self->profilingData.ray.triangles++;)

                let mut t: f32 = 0.;
                //if (Intersect_RayTriangle_Barycentric(ray, triangle, tEpsilon, &t)) {
                if Intersect_RayTriangle_Moller1(&mut ray, triangle, &mut t) {
                    //if (Intersect_RayTriangle_Moller2(ray, triangle, &t)) {
                    //if (Intersect_RayTriangle_Badouel(ray, triangle, tEpsilon, &t)) {
                    if !hit || t < *tHit {
                        hit = true;
                        *tHit = t;
                    }
                }
                i = i.wrapping_add(1);
            }

            if hit {
                break;
            }

            if rayStack.is_empty() {
                break;
            }

            let d = rayStack.pop().unwrap();
            nodeRef = d.nodeRef;
            ray.tMin = d.tMin as f64;
            ray.tMax = d.tMax as f64;
            depth = d.depth;
        }
    }

    rayStack.clear();
    // BSP_PROFILE (
    //     self->profilingData.ray.count++;
    //     self->profilingData.ray.depth += maxDepth;
    // )

    hit
}

#[no_mangle]
pub unsafe extern "C" fn BSP_IntersectLineSegment(
    this: &mut BSP,
    lineSegment: &LineSegment,
    pHit: &mut Vec3,
) -> bool {
    let mut ray: Ray = Ray {
        p: lineSegment.p0,
        dir: lineSegment.p1.as_dvec3() - lineSegment.p0.as_dvec3(),
        tMin: 0.0,
        tMax: 1.0,
    };
    let mut t: f32 = 0.;
    if BSP_IntersectRay(this, &mut ray, &mut t) {
        let mut positionHit = Position::ZERO;
        Ray_GetPoint(&ray, t as f64, &mut positionHit);
        *pHit = positionHit.as_vec3();
        true
    } else {
        false
    }
}

#[no_mangle]
pub static mut nodeStack: Vec<Delay> = Vec::new();

#[no_mangle]
pub unsafe extern "C" fn BSP_IntersectSphere(
    this: &mut BSP,
    sphere: &Sphere,
    pHit: &mut Vec3,
) -> bool {
    // Assert(SPHERE_INTERSECTION_EPSILON > PLANE_THICKNESS_EPSILON);

    let mut nodeRef: BSPNodeRef = this.rootNode;
    let mut hit: bool = false;
    let mut depth: i32 = 0;
    let mut maxDepth: i32 = 0;

    loop {
        maxDepth = i32::max(depth, maxDepth);

        if nodeRef.index >= 0 {
            let node: &mut BSPNode = &mut this.nodes[nodeRef.index as usize];
            // BSP_PROFILE(self->profilingData.sphere.nodes++;)

            let dist: f32 = Vec3::dot((*node).plane.n, (*sphere).p) - (*node).plane.d;
            if dist as f64 > (*sphere).r as f64 + 2.0f64 * 1e-4f64 {
                /* Entirely in front half-space */
                nodeRef = (*node).child[FrontIndex as usize];
            } else if (dist as f64) < -((*sphere).r as f64 + 2.0f64 * 1e-4f64) {
                /* Entirely in back half-space */
                nodeRef = (*node).child[BackIndex as usize];
            } else {
                /* Straddling the thick plane */
                let d: Delay = Delay {
                    nodeRef: (*node).child[BackIndex as usize],
                    depth,
                };
                nodeStack.push(d);
                nodeRef = (*node).child[FrontIndex as usize];
            }

            depth += 1;
        } else {
            let leafIndex = -nodeRef.index;
            // BSP_PROFILE(self->profilingData.sphere.leaves++;)

            let mut i: u8 = 0 as u8;
            while (i as i32) < nodeRef.triangleCount as i32 {
                let triangle = &this.triangles[leafIndex as usize + i as usize];
                // BSP_PROFILE(self->profilingData.sphere.triangles++;)

                let mut pHit2 = Vec3::ZERO;
                if Intersect_SphereTriangle(sphere, triangle, &mut pHit2) {
                    hit = true;
                    *pHit = pHit2;
                    break;
                }
                i = i.wrapping_add(1);
            }

            if hit {
                break;
            }

            if nodeStack.is_empty() {
                break;
            }

            let d = nodeStack.pop().unwrap();
            nodeRef = d.nodeRef;
            depth = d.depth;
        }
    }

    nodeStack.clear();
    // BSP_PROFILE (
    //     self->profilingData.sphere.count++;
    //     self->profilingData.sphere.depth += maxDepth;
    // )

    hit
}

// const DEFAULT_TRIANGLE_SPLIT_COST: f32 = 0.85;
// const LEAF_TRIANGLE_COUNT: i32 = 12;

#[no_mangle]
pub static PolygonFlag_None: PolygonFlag = (0 << 0) as PolygonFlag;

#[no_mangle]
pub static PolygonFlag_InvalidFaceSplit: PolygonFlag = (1 << 0) as PolygonFlag;

#[no_mangle]
pub static PolygonFlag_InvalidDecompose: PolygonFlag = (1 << 1) as PolygonFlag;

#[no_mangle]
pub static PolygonFlag_InvalidEdgeSplit: PolygonFlag = (1 << 2) as PolygonFlag;

unsafe extern "C" fn BSPBuild_ScoreSplitPlane(
    nodeData: *mut BSPBuildNodeData,
    plane: Plane,
    k: f32,
) -> f32 {
    /* The bigger k is, the more we penalize polygon splitting */
    // Assert(k >= 0.0f && k <= 1.0f);

    let mut numInFront: i32 = 0;
    let mut numBehind: i32 = 0;
    let mut numStraddling: i32 = 0;

    for polygon in (*nodeData).polygons.iter() {
        match Plane_ClassifyPolygon(&plane, &polygon.inner) {
            PolygonClassification::Coplanar | PolygonClassification::Behind => {
                numBehind += 1;
            }
            PolygonClassification::InFront => {
                numInFront += 1;
            }
            PolygonClassification::Straddling => {
                numStraddling += 1;
            }
        }
    }

    //k*numStraddling + (1.0f - k)*Abs(numInFront - numBehind);
    Lerp(
        f64::abs((numInFront - numBehind) as f64) as f64,
        numStraddling as f64,
        k as f64,
    ) as f32
}

unsafe extern "C" fn BSPBuild_ChooseSplitPlane(
    bsp: *mut BSPBuild,
    nodeData: *mut BSPBuildNodeData,
    splitPlane: *mut Plane,
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

    let maxDepth: f32 = 1000.0f32;
    let biasedDepth: f32 = (*nodeData).depth as f32 - 100.0f32;
    let t: f32 = f64::max((biasedDepth / maxDepth) as f64, 0.0f64) as f32;
    let k: f32 = Lerp(0.85f64, 0.25f64, t as f64) as f32;

    let mut bestScore: f32 = f32::MAX;
    let mut bestPlane: Plane = Plane {
        n: Vec3::ZERO,
        d: 0.,
    };
    let mut bestPolygon: *mut PolygonEx = std::ptr::null_mut();
    let mut numToCheck: i32 = 10;

    let polygonsLen: i32 = (*nodeData).polygons.len() as i32;
    if (*nodeData).validPolygonCount > 0 {
        /* Simply score split planes using polygon faces */
        numToCheck = i32::min(numToCheck, (*nodeData).validPolygonCount);
        let mut i: i32 = 0;
        while i < numToCheck {
            let mut polygonIndex: i32 =
                (RNG_Get32(&mut *(*bsp).rng)).wrapping_rem(polygonsLen as u32) as i32;

            /* OPTIMIZE: This search is duuuuuumb. Maybe We should swap invalid
             *           polygons to the end of the list so never have to search.
             */
            let mut j: i32 = 0;
            while j < polygonsLen {
                let polygon: *mut PolygonEx = &mut (*nodeData).polygons[polygonIndex as usize];

                if (*polygon).flags as i32 & PolygonFlag_InvalidFaceSplit as i32 == 0 {
                    let mut plane: Plane = Plane {
                        n: Vec3::ZERO,
                        d: 0.,
                    };
                    Polygon_ToPlane(&mut (*polygon).inner, &mut plane);
                    let score: f32 = BSPBuild_ScoreSplitPlane(nodeData, plane, k);

                    if score < bestScore {
                        bestScore = score;
                        bestPlane = plane;
                        bestPolygon = polygon;
                    }
                    break;
                }

                polygonIndex = (polygonIndex + 1) % polygonsLen;
                j += 1;
            }
            i += 1;
        }

        if !bestPolygon.is_null() {
            (*bestPolygon).flags =
                ((*bestPolygon).flags as i32 | PolygonFlag_InvalidFaceSplit as i32) as PolygonFlag;
            // CHECK2(Polygon_GetCentroid((Polygon*) bestPolygon, &node->planeCenter);)
        }
    } else if polygonsLen > 0 {
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

        let mut splitFound: bool = false;

        /* Try to split any polygons with more than 1 triangle */
        if !splitFound {
            let mut polygonIndex: i32 =
                (RNG_Get32(&mut *(*bsp).rng)).wrapping_rem(polygonsLen as u32) as i32;
            for _ in 0..polygonsLen {
                let polygon: *mut PolygonEx = &mut (*nodeData).polygons[polygonIndex as usize];
                if (*polygon).flags as i32 & PolygonFlag_InvalidDecompose as i32 != 0 {
                    continue;
                }

                let v: &Vec<Vec3> = &(*polygon).inner.vertices;
                for j in 2..(v.len() - 1) {
                    let edge: Vec3 = v[0] - v[j];
                    let mid: Vec3 = Vec3::lerp(v[0], v[j], 0.5f32);

                    /* TODO : Maybe just save the plane with polygon while build so they're only calculated once? */
                    let mut polygonPlane: Plane = Plane {
                        n: Vec3::ZERO,
                        d: 0.,
                    };
                    Polygon_ToPlane(&mut (*polygon).inner, &mut polygonPlane);

                    let mut plane: Plane = Plane {
                        n: Vec3::ZERO,
                        d: 0.,
                    };
                    plane.n = Vec3::cross(edge, polygonPlane.n).normalize();
                    plane.d = Vec3::dot(plane.n, mid);

                    /* TODO : Proper scoring? */
                    if Plane_ClassifyPolygon(&mut plane, &mut (*polygon).inner)
                        == PolygonClassification::Straddling
                    {
                        splitFound = true;

                        bestScore = 0.0;
                        bestPlane = plane;
                        bestPolygon = polygon;
                        // CHECK2(node->planeCenter = mid;)
                        break;
                    } else {
                        /* This is possible because we don't fully handle slivers. There's
                         * nothing stopping a triangle from being thinner than
                         * PLANE_THICKNESS_EPSILON. */
                        (*polygon).flags = ((*polygon).flags as i32
                            | PolygonFlag_InvalidDecompose as i32)
                            as PolygonFlag;
                    }
                    //if (--numToCheck == 0) break;
                }

                if splitFound {
                    break;
                }
                //if (numToCheck == 0) break;
                polygonIndex = (polygonIndex + 1) % polygonsLen;
            }

            if splitFound {
                (*bestPolygon).flags = ((*bestPolygon).flags as i32
                    | PolygonFlag_InvalidDecompose as i32)
                    as PolygonFlag;
            }
        }

        /* Try splitting along a polygon edge */
        if !splitFound {
            let mut polygonIndex: i32 =
                (RNG_Get32(&mut *(*bsp).rng)).wrapping_rem(polygonsLen as u32) as i32;
            for _ in 0..polygonsLen {
                let polygon: *mut PolygonEx = &mut (*nodeData).polygons[polygonIndex as usize];
                if (*polygon).flags as i32 & PolygonFlag_InvalidEdgeSplit as i32 != 0 {
                    continue;
                }

                let mut polygonPlane: Plane = Plane {
                    n: Vec3 {
                        x: 0.,
                        y: 0.,
                        z: 0.,
                    },
                    d: 0.,
                };
                Polygon_ToPlane(&mut (*polygon).inner, &mut polygonPlane);

                let v = &mut (*polygon).inner.vertices;
                let mut vPrev: Vec3 = v[(v.len() - 1) as usize];
                for j in 0..v.len() {
                    let vCur: Vec3 = v[j];
                    let edge: Vec3 = vCur - vPrev;
                    let mid: Vec3 = Vec3::lerp(vPrev, vCur, 0.5f32);

                    let mut plane: Plane = Plane {
                        n: Vec3 {
                            x: 0.,
                            y: 0.,
                            z: 0.,
                        },
                        d: 0.,
                    };
                    plane.n = Vec3::cross(edge, polygonPlane.n).normalize();
                    plane.d = Vec3::dot(plane.n, mid);

                    let score: f32 = BSPBuild_ScoreSplitPlane(nodeData, plane, 0.0f32);
                    if score < bestScore {
                        splitFound = true;

                        bestPolygon = polygon;
                        bestScore = score;
                        bestPlane = plane;
                        // CHECK2(node->planeCenter = mid;)
                    }

                    vPrev = vCur;
                    numToCheck -= 1;
                    if numToCheck == 0 {
                        break;
                    }
                }

                if numToCheck == 0 {
                    break;
                }
                polygonIndex = (polygonIndex + 1) % polygonsLen;
            }

            if splitFound {
                (*bestPolygon).flags = ((*bestPolygon).flags as i32
                    | PolygonFlag_InvalidEdgeSplit as i32)
                    as PolygonFlag;
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

    if bestScore < f32::MAX {
        *splitPlane = bestPlane;
        true
    } else {
        false
    }
}

#[inline]
unsafe extern "C" fn BSPBuild_AppendPolygon(
    nodeData: *mut BSPBuildNodeData,
    polygon: *const PolygonEx,
) {
    //if (nodeData->triangleCount == 0) {
    //  Vec3f v0 = ArrayList_Get(polygon->vertices, 0);
    //  nodeData->boundingBox.lower = v0;
    //  nodeData->boundingBox.upper = v0;
    //}
    //ArrayList_ForEach(polygon->vertices, Vec3f, v) {
    //  Box3f_Add(&nodeData->boundingBox, *v);
    //}

    (*nodeData).triangleCount += (*polygon).inner.vertices.len() as i32 - 2;
    (*nodeData).validPolygonCount +=
        ((*polygon).flags as i32 & PolygonFlag_InvalidFaceSplit as i32 == 0) as i32;
    (*nodeData).polygons.push((*polygon).clone())
}

unsafe extern "C" fn BSPBuild_CreateNode(
    bsp: *mut BSPBuild,
    nodeData: *mut BSPBuildNodeData,
) -> *mut BSPBuildNode {
    /* NOTE: This will free the polygons being passed in! This is to prevent all
     *        the temporary allocations from overlapping. */

    /* NOTE: Coplanar polygons are considered to be behind the plane and will
     *        therefore lead to collisions. It seems preferable to push objects
     *        very slightly outside of each other during a collision, rather than
     *        letting them very slightly overlap. */

    // Assert(nodeData->depth < 1 << 8*sizeof(nodeData->depth));

    let node = MemNewZero!(BSPBuildNode);
    // CHECK2(node->id = bsp->nextNodeID++;)

    let mut splitPlane: Plane = Plane {
        n: Vec3::ZERO,
        d: 0.,
    };

    let mut makeLeaf: bool = false;
    makeLeaf = makeLeaf as i32 != 0 || (*nodeData).triangleCount <= 12;
    makeLeaf = makeLeaf as i32 != 0 || !BSPBuild_ChooseSplitPlane(bsp, nodeData, &mut splitPlane);

    if makeLeaf {
        if (*nodeData).triangleCount != 0 {
            (*bsp).leafCount += 1;
        }
        (*bsp).triangleCount += (*nodeData).triangleCount;

        (*node).polygons = (*nodeData).polygons.clone();
        return node;
    }

    (*bsp).nodeCount += 1;

    let polygonsLen = (*nodeData).polygons.len();

    let mut backNodeData: BSPBuildNodeData = BSPBuildNodeData {
        polygons: Vec::new(),
        validPolygonCount: 0,
        triangleCount: 0,
        depth: 0,
    };
    backNodeData.polygons.reserve(polygonsLen);
    backNodeData.depth = ((*nodeData).depth as i32 + 1) as u16;

    let mut frontNodeData: BSPBuildNodeData = BSPBuildNodeData {
        polygons: Vec::new(),
        validPolygonCount: 0,
        triangleCount: 0,
        depth: 0,
    };
    frontNodeData.polygons.reserve(polygonsLen);
    frontNodeData.depth = ((*nodeData).depth as i32 + 1) as u16;

    for polygon in (*nodeData).polygons.iter_mut() {
        let classification = Plane_ClassifyPolygon(&mut splitPlane, &polygon.inner);
        match classification {
            PolygonClassification::Coplanar => {
                (*polygon).flags =
                    ((*polygon).flags as i32 | PolygonFlag_InvalidFaceSplit as i32) as PolygonFlag;
            }
            PolygonClassification::Behind => {
                BSPBuild_AppendPolygon(&mut backNodeData, polygon);
            }
            PolygonClassification::InFront => {
                BSPBuild_AppendPolygon(&mut frontNodeData, polygon);
            }
            PolygonClassification::Straddling => {
                let mut backPart = PolygonEx {
                    inner: Polygon {
                        vertices: Vec::new(),
                    },
                    flags: 0,
                };
                backPart.flags = (*polygon).flags;

                let mut frontPart = PolygonEx {
                    inner: Polygon {
                        vertices: Vec::new(),
                    },
                    flags: 0,
                };
                frontPart.flags = (*polygon).flags;

                Polygon_SplitSafe(
                    &polygon.inner,
                    splitPlane,
                    &mut backPart.inner,
                    &mut frontPart.inner,
                );
                BSPBuild_AppendPolygon(&mut backNodeData, &mut backPart);
                BSPBuild_AppendPolygon(&mut frontNodeData, &mut frontPart);

                (*polygon).inner.vertices.clear();
            }
        }
    }
    (*nodeData).polygons.clear();

    (*node).plane = splitPlane;
    (*node).child[BackIndex as usize] = BSPBuild_CreateNode(bsp, &mut backNodeData);
    (*node).child[FrontIndex as usize] = BSPBuild_CreateNode(bsp, &mut frontNodeData);

    // CHECK2 (
    //     node->child[BackIndex] ->parent = node;
    //     node->child[FrontIndex]->parent = node;
    // )

    node
}

unsafe extern "C" fn BSPBuild_OptimizeTree(
    this: &mut BSP,
    buildNode: *mut BSPBuildNode,
) -> BSPNodeRef {
    if !((*buildNode).child[BackIndex as usize]).is_null()
        || !((*buildNode).child[FrontIndex as usize]).is_null()
    {
        /* Node */
        // Assert(ArrayList_GetSize(self->nodes) < ArrayList_GetCapacity(self->nodes));

        let dummy: BSPNode = BSPNode {
            plane: Plane {
                n: Vec3::ZERO,
                d: 0.,
            },
            child: [BSPNodeRef {
                index: 0,
                triangleCount: 0,
            }; 2],
        };
        let nodeIndex: i32 = this.nodes.len() as i32;
        this.nodes.push(dummy);
        let node = this.nodes.last_mut().unwrap() as *mut BSPNode;

        (*node).plane = (*buildNode).plane;
        (*node).child[BackIndex as usize] =
            BSPBuild_OptimizeTree(this, (*buildNode).child[BackIndex as usize]);
        (*node).child[FrontIndex as usize] =
            BSPBuild_OptimizeTree(this, (*buildNode).child[FrontIndex as usize]);

        let result: BSPNodeRef = BSPNodeRef {
            index: nodeIndex,
            triangleCount: 0 as u8,
        };
        result
    } else {
        /* Leaf */
        if (*buildNode).polygons.is_empty() {
            return this.emptyLeaf;
        }

        let leafIndex = this.triangles.len();

        for polygon in (*buildNode).polygons.iter() {
            // Assert(
            //     ArrayList_GetSize(self->triangles) +
            //     ArrayList_GetSize(polygon->vertices) - 2
            //     <= ArrayList_GetCapacity(self->triangles)
            // );
            Polygon_ConvexToTriangles(&(*polygon).inner, &mut this.triangles);
        }

        let leafLen: u8 = (this.triangles.len() - leafIndex) as u8;
        BSPNodeRef {
            index: -(leafIndex as i32),
            triangleCount: leafLen,
        }
    }
}

unsafe extern "C" fn BSPBuild_FreeNode(node: *mut BSPBuildNode) {
    if !((*node).child[BackIndex as usize]).is_null()
        || !((*node).child[FrontIndex as usize]).is_null()
    {
        BSPBuild_FreeNode((*node).child[BackIndex as usize]);
        BSPBuild_FreeNode((*node).child[FrontIndex as usize]);
    }
    MemDelete!(node);
}

/*
BSP_PROFILE (
static void BSPBuild_AnalyzeTree (BSP* self, Mesh* mesh, BSPNodeRef nodeRef, int32 depth) {
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

#[no_mangle]
pub unsafe extern "C" fn BSP_Create(mesh: &mut Mesh) -> *mut BSP {
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

    let this = MemNewZero!(BSP);

    let indexLen: i32 = Mesh_GetIndexCount(mesh);
    let indexData: *mut i32 = Mesh_GetIndexData(mesh);
    let vertexData: *mut Vertex = Mesh_GetVertexData(mesh);

    /* TODO : Implement some form of soft abort when the incoming mesh is bad. */
    // CHECK2 (
    //     if (Mesh_Validate(mesh) != Error_None) return 0;
    // )

    let mut nodeData: BSPBuildNodeData = BSPBuildNodeData {
        polygons: Vec::new(),
        validPolygonCount: 0,
        triangleCount: 0,
        depth: 0,
    };
    nodeData.triangleCount = indexLen / 3;
    nodeData.validPolygonCount = indexLen / 3;

    nodeData.polygons.reserve(nodeData.triangleCount as usize);
    for i in (0..indexLen).step_by(3) {
        let i0: i32 = *indexData.offset((i + 0) as isize);
        let i1: i32 = *indexData.offset((i + 1) as isize);
        let i2: i32 = *indexData.offset((i + 2) as isize);
        let v0: Vec3 = (*vertexData.offset(i0 as isize)).p;
        let v1: Vec3 = (*vertexData.offset(i1 as isize)).p;
        let v2: Vec3 = (*vertexData.offset(i2 as isize)).p;

        nodeData.polygons.push(PolygonEx {
            inner: Polygon {
                vertices: vec![v0, v1, v2],
            },
            flags: 0,
        });
    }

    /* Build */
    let mut bspBuild: BSPBuild = BSPBuild {
        rootNode: std::ptr::null_mut(),
        rng: RNG_Create(1235),
        nodeCount: 0,
        leafCount: 0,
        triangleCount: 0,
    };
    bspBuild.rootNode = BSPBuild_CreateNode(&mut bspBuild, &mut nodeData);

    /* Optimize */
    let nullLeaf: Triangle = Triangle {
        vertices: [Vec3::ZERO; 3],
    };
    (*this)
        .triangles
        .reserve((bspBuild.triangleCount + 2) as usize);
    (*this).triangles.push(nullLeaf);
    (*this).triangles.push(nullLeaf);
    (*this).emptyLeaf.index = -EmptyLeafIndex;
    (*this).emptyLeaf.triangleCount = 0;

    let nullNode: BSPNode = BSPNode {
        plane: Plane {
            n: Vec3::ZERO,
            d: 0.,
        },
        child: [BSPNodeRef {
            index: 0,
            triangleCount: 0,
        }; 2],
    };
    (*this).nodes.reserve((bspBuild.nodeCount + 1) as usize);
    (*this).nodes.push(nullNode);
    (*this).rootNode = BSPBuild_OptimizeTree(&mut *this, bspBuild.rootNode);
    // #if BSP_PROFILE && CHECK_LEVEL >= 2
    //     self->profilingData.oversizedNodes = bspBuild.oversizedNodes;
    //     self->profilingData.avgOversizeAmount = bspBuild.avgOversizeAmount;
    //     if (bspBuild.oversizedNodes > 0) {
    //     Warn("BSP_Create: Created %i oversized leaves with an average excess of %.1f triangles.", bspBuild.oversizedNodes, bspBuild.avgOversizeAmount);
    //     }
    // #endif

    BSPBuild_FreeNode(bspBuild.rootNode);
    RNG_Free(Some(bspBuild.rng));

    // Assert(ArrayList_GetSize(self->nodes)     == ArrayList_GetCapacity(self->nodes));
    // Assert(ArrayList_GetSize(self->triangles) == ArrayList_GetCapacity(self->triangles));
    // BSP_PROFILE(BSPBuild_AnalyzeTree(self, mesh, self->rootNode, 0);)

    this
}

#[no_mangle]
pub unsafe extern "C" fn BSP_Free(this: *mut BSP) {
    MemDelete!(this);
}

#[no_mangle]
pub unsafe extern "C" fn BSPDebug_GetNode(
    this: &mut BSP,
    nodeRef: BSPNodeRef,
    relationship: BSPNodeRel,
) -> BSPNodeRef {
    if nodeRef.index == 0 {
        return this.rootNode;
    }

    let mut node: *mut BSPNode = std::ptr::null_mut();
    if nodeRef.index > 0 {
        node = &mut this.nodes[nodeRef.index as usize];
    }

    let mut newNode: BSPNodeRef = BSPNodeRef {
        index: 0,
        triangleCount: 0,
    };
    if relationship == BSPNodeRel_Parent {
        if nodeRef.index != 0 {
            for i in 0..(this.nodes.len() as i32) {
                let nodeToCheck: &mut BSPNode = &mut this.nodes[i as usize];
                if (*nodeToCheck).child[BackIndex as usize].index == nodeRef.index {
                    newNode.index = i;
                    break;
                } else if (*nodeToCheck).child[FrontIndex as usize].index == nodeRef.index {
                    newNode.index = i;
                    break;
                }
            }
        }
    } else if relationship == BSPNodeRel_Back {
        if !node.is_null() {
            newNode = (*node).child[BackIndex as usize];
        }
    } else if relationship == BSPNodeRel_Front {
        if !node.is_null() {
            newNode = (*node).child[FrontIndex as usize];
        }
    } else {
        panic!("BSPDebug_GetNode: Unhandled case: {}", relationship as i32,)
    }

    if newNode.index != 0 {
        newNode
    } else {
        nodeRef
    }
}

#[no_mangle]
pub unsafe extern "C" fn BSPDebug_DrawNode(this: &mut BSP, nodeRef: BSPNodeRef) {
    // Assert(nodeRef.index);

    if nodeRef.index > 0 {
        BSPDebug_DrawNode(
            this,
            this.nodes[nodeRef.index as usize].child[BackIndex as usize],
        );
        BSPDebug_DrawNode(
            this,
            this.nodes[nodeRef.index as usize].child[FrontIndex as usize],
        );
    } else {
        let leafIndex = -nodeRef.index;
        for i in 0..nodeRef.triangleCount {
            let triangle: *mut Triangle = &mut this.triangles[leafIndex as usize + i as usize];
            Draw_Poly3(((*triangle).vertices).as_mut_ptr(), 3);
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn BSPDebug_DrawNodeSplit(this: &mut BSP, nodeRef: BSPNodeRef) {
    // Assert(nodeRef.index);

    RenderState_PushBlendMode(1);
    RenderState_PushCullFace(1);
    RenderState_PushDepthTest(true);
    RenderState_PushWireframe(true);

    if nodeRef.index > 0 {
        let node: *const BSPNode = &this.nodes[nodeRef.index as usize] as *const _;

        /* Back */
        Draw_Color(0.5f32, 0.3f32, 0.3f32, 0.4f32);
        BSPDebug_DrawNode(this, (*node).child[BackIndex as usize]);

        /* Front */
        Draw_Color(0.3f32, 0.5f32, 0.3f32, 0.4f32);
        BSPDebug_DrawNode(this, (*node).child[FrontIndex as usize]);

        /* Plane */
        let origin: Vec3 = Vec3::new(0., 0., 0.);
        let t: f32 = Vec3::dot((*node).plane.n, origin) - (*node).plane.d;
        let mut closestPoint = origin - ((*node).plane.n * t);
        RenderState_PushWireframe(false);
        Draw_Color(0.3f32, 0.5f32, 0.3f32, 0.4f32);
        Draw_Plane(&closestPoint, &(*node).plane.n, 2.0f32);
        Draw_Color(0.5f32, 0.3f32, 0.3f32, 0.4f32);
        let mut neg: Vec3 = (*node).plane.n * -1.0f32;
        Draw_Plane(&mut closestPoint, &mut neg, 2.0f32);
        RenderState_PopWireframe();
    } else {
        /* Leaf */
        Draw_Color(0.5f32, 0.5f32, 0.3f32, 0.4f32);
        BSPDebug_DrawNode(this, nodeRef);
    }

    RenderState_PopWireframe();
    RenderState_PopDepthTest();
    RenderState_PopCullFace();
    RenderState_PopBlendMode();
}

#[no_mangle]
pub unsafe extern "C" fn BSPDebug_DrawLineSegment(bsp: &mut BSP, lineSegment: &mut LineSegment, eye: &Position) {
    let mut pHit = Vec3::ZERO;
    if BSP_IntersectLineSegment(bsp, lineSegment, &mut pHit) {
        Draw_Color(0.0f32, 1.0f32, 0.0f32, 0.1f32);
        Draw_Line3(
            &(*lineSegment).p0.relative_to(*eye),
            &Position::from_vec(pHit).relative_to(*eye)
        );

        Draw_Color(1.0f32, 0.0f32, 0.0f32, 1.0f32);
        Draw_Line3(
            &Position::from_vec(pHit).relative_to(*eye),
            &(*lineSegment).p1.relative_to(*eye)
        );

        Draw_PointSize(5.0f32);
        Draw_Point3(pHit.x, pHit.y, pHit.z);
    } else {
        Draw_Color(0.0f32, 1.0f32, 0.0f32, 1.0f32);
        Draw_Line3(
            &(*lineSegment).p0.relative_to(*eye),
            &(*lineSegment).p1.relative_to(*eye)
        );
    };
}

#[no_mangle]
pub unsafe extern "C" fn BSPDebug_DrawSphere(this: &mut BSP, sphere: &mut Sphere) {
    let mut pHit = Vec3::ZERO;
    if BSP_IntersectSphere(this, sphere, &mut pHit) {
        RenderState_PushWireframe(false);
        Draw_Color(1.0f32, 0.0f32, 0.0f32, 0.3f32);
        Draw_Sphere(&mut sphere.p, sphere.r);
        RenderState_PopWireframe();

        Draw_Color(1.0f32, 0.0f32, 0.0f32, 1.0f32);
        Draw_Sphere(&mut sphere.p, sphere.r);

        RenderState_PushDepthTest(false);
        Draw_PointSize(8.0f32);
        Draw_Point3(pHit.x, pHit.y, pHit.z);
        RenderState_PopDepthTest();
    } else {
        RenderState_PushWireframe(false);
        Draw_Color(0.0f32, 1.0f32, 0.0f32, 0.3f32);
        Draw_Sphere(&mut sphere.p, sphere.r);
        RenderState_PopWireframe();

        Draw_Color(0.0f32, 1.0f32, 0.0f32, 1.0f32);
        Draw_Sphere(&mut sphere.p, sphere.r);
    };
}

// static void BSPDebug_PrintProfilingData (BSP* self, BSPDebug_IntersectionData* data, double totalTime) {
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

#[no_mangle]
pub extern "C" fn BSPDebug_PrintRayProfilingData(_this: &mut BSP, _totalTime: f64) {
    // #if ENABLE_BSP_PROFILING
    //   BSPDebug_PrintProfilingData(self, &self->profilingData.ray, totalTime);
    // #else
    warn!("BSP_PrintRayProfilingData: BSP profiling is not enabled. Set ENABLE_BSP_PROFILING to enable this function.");
}

#[no_mangle]
pub extern "C" fn BSPDebug_PrintSphereProfilingData(_this: &mut BSP, _totalTime: f64) {
    // #if ENABLE_BSP_PROFILING
    //     BSPDebug_PrintProfilingData(self, &self->profilingData.sphere, totalTime);
    // #else
    warn!("BSP_PrintSphereProfilingData: BSP profiling is not enabled. Set ENABLE_BSP_PROFILING to enable this function.");
}

#[no_mangle]
pub unsafe extern "C" fn BSPDebug_GetIntersectSphereTriangles(
    this: &mut BSP,
    sphere: &mut Sphere,
    sphereProf: &mut IntersectSphereProfiling,
) -> bool {
    // Assert(SPHERE_INTERSECTION_EPSILON > PLANE_THICKNESS_EPSILON);

    let mut nodeRef: BSPNodeRef = this.rootNode;
    let mut hit: bool = false;
    let mut depth: i32 = 0;
    let mut maxDepth: i32 = 0;

    loop {
        maxDepth = i32::max(depth, maxDepth);

        if nodeRef.index >= 0 {
            let node: &mut BSPNode = &mut this.nodes[nodeRef.index as usize];
            (*sphereProf).nodes += 1;

            let dist: f32 = Vec3::dot((*node).plane.n, (*sphere).p) - (*node).plane.d;
            if dist as f64 > (*sphere).r as f64 + 2.0f64 * 1e-4f64 {
                /* Entirely in front half-space */
                nodeRef = (*node).child[FrontIndex as usize];
            } else if (dist as f64) < -((*sphere).r as f64 + 2.0f64 * 1e-4f64) {
                /* Entirely in back half-space */
                nodeRef = (*node).child[BackIndex as usize];
            } else {
                /* Straddling the thick plane */
                let d: Delay = Delay {
                    nodeRef: (*node).child[BackIndex as usize],
                    depth,
                };
                nodeStack.push(d);
                nodeRef = (*node).child[FrontIndex as usize];
            }

            depth += 1;
        } else {
            let leafIndex = -nodeRef.index;
            (*sphereProf).leaves += 1;

            for i in 0..nodeRef.triangleCount {
                let triangle = &mut this.triangles[leafIndex as usize + i as usize];
                (*sphereProf).triangles += 1;

                let mut pHit2 = Vec3::ZERO;
                if Intersect_SphereTriangle(sphere, triangle, &mut pHit2) {
                    let t: TriangleTest = TriangleTest {
                        triangle,
                        hit: true,
                    };
                    (*sphereProf).triangleTests.push(t);
                    hit = true;
                    break;
                }

                let t: TriangleTest = TriangleTest {
                    triangle,
                    hit: false,
                };
                (*sphereProf).triangleTests.push(t);
            }

            if hit {
                break;
            }

            if nodeStack.is_empty() {
                break;
            }

            let d: Delay = nodeStack.pop().unwrap();
            nodeRef = d.nodeRef;
            depth = d.depth;
        }
    }

    nodeStack.clear();

    hit
}

#[no_mangle]
pub unsafe extern "C" fn BSPDebug_GetLeaf(this: &mut BSP, leafIndex: i32) -> BSPNodeRef {
    let mut index: i32 = -1;
    for node in this.nodes.iter() {
        if (*node).child[0].index < 0 {
            let prevIndex = index;
            index += 1;
            if prevIndex == leafIndex {
                return (*node).child[0];
            }
        }
        if (*node).child[1].index < 0 {
            let prevIndex = index;
            index += 1;
            if prevIndex == leafIndex {
                return (*node).child[1];
            }
        }
    }
    BSPNodeRef {
        index: RootNodeIndex,
        triangleCount: 0 as u8,
    }
}
