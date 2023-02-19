// struct Vec2<T> {
//     x: T,
//     y: T,
// }
//
// impl Vec2<T> {
//     fn create(x: T, y: T) -> Vec2<T> {
//         Vec2 { x, y }
//     }
//
//     fn abs(&self) -> Vec2<T> {
//         Vec2 { }
//
//     /*
//
// #define Vec2_Abs(T, prefix) inline T T##_Abs(T v) { \
//   T self = { Abs##prefix(v.x), Abs##prefix(v.y) }; return self; }
//
// Vec2_Abs(Vec2i, i)
// Vec2_Abs(Vec2f, f)
// Vec2_Abs(Vec2d,)
//
// #define Vec2_IAbs(T, prefix) inline void T##_IAbs(T* v) { \
//   v->x = Abs##prefix(v->x); v->y = Abs##prefix(v->y); }
//
// Vec2_IAbs(Vec2i, i)
// Vec2_IAbs(Vec2f, f)
// Vec2_IAbs(Vec2d,)
//
// #define Vec2_Equal(T) inline bool T##_Equal(T a, T b) { \
//   bool self = a.x == b.x && a.y == b.y; \
//   return self; }
//
// Vec2_Equal(Vec2i)
// Vec2_Equal(Vec2f)
// Vec2_Equal(Vec2d)
//
// #define Vec2_Length(T, sub, prefix) inline sub T##_Length(T v) { \
//   return Sqrt##prefix(v.x * v.x + v.y * v.y); }
//
// Vec2_Length(Vec2f, float, f)
// Vec2_Length(Vec2d, double,)
//
// #define Vec2_LengthSquared(T, sub) inline sub T##_LengthSquared(T v) { \
//   return v.x * v.x + v.y * v.y; }
//
// Vec2_LengthSquared(Vec2f, float)
// Vec2_LengthSquared(Vec2d, double)
//
// #define Vec2_Normalize(T, sub) inline T T##_Normalize(T v) { \
//   sub l = T##_Length(v); \
//   T self = { v.x / l, v.y / l }; return self; }
//
// Vec2_Normalize(Vec2f, float)
// Vec2_Normalize(Vec2d, double)
//
// #define Vec2_Dot(T, sub) inline sub T##_Dot(T a, T b) { \
//   return a.x * b.x + a.y * b.y; }
//
// Vec2_Dot(Vec2i, int)
// Vec2_Dot(Vec2f, float)
// Vec2_Dot(Vec2d, double)
//
// #define Vec2_Validate(T, sub, prefix) inline Error T##_Validate(T v) { \
//   Error e = Error_None; \
//   e |= Float_Validate##prefix(v.x); \
//   e |= Float_Validate##prefix(v.y); \
//   return e; }
//
// Vec2_Validate(Vec2f, float, f)
// Vec2_Validate(Vec2d, double,)
//
// #define Vec2_ToString(T, sub, fmt) static inline cstr T##_ToString(T* v) { \
//   static char buffer[512]; \
//   snprintf(buffer, (size_t) Array_GetSize(buffer), \
//     "(" fmt ", " fmt ")", v->x, v->y); \
//   return buffer; }
//      */
// }
//
// // Operators
//
//     /*
// #define Vec2_Add(T) inline T T##_Add(T a, T b) { \
//   T self = { a.x + b.x, a.y + b.y }; return self; }
// #define Vec2_Div(T) inline T T##_Div(T a, T b) { \
//   T self = { a.x / b.x, a.y / b.y }; return self; }
// #define Vec2_Mul(T) inline T T##_Mul(T a, T b) { \
//   T self = { a.x * b.x, a.y * b.y }; return self; }
// #define Vec2_Sub(T) inline T T##_Sub(T a, T b) { \
//   T self = { a.x - b.x, a.y - b.y }; return self; }
//
// #define Vec2_IAdd(T) inline void T##_IAdd(T* a, T b) { \
//   a->x += b.x; a->y += b.y; }
// #define Vec2_IDiv(T) inline void T##_IDiv(T* a, T b) { \
//   a->x /= b.x; a->y /= b.y; }
// #define Vec2_IMul(T) inline void T##_IMul(T* a, T b) { \
//   a->x *= b.x; a->y *= b.y; }
// #define Vec2_ISub(T) inline void T##_ISub(T* a, T b) { \
//   a->x -= b.x; a->y -= b.y; }
//
// #define Vec2_Adds(T, sub) inline T T##_Adds(T a, sub b) { \
//   T self = { a.x + b, a.y + b }; return self; }
// #define Vec2_Divs(T, sub) inline T T##_Divs(T a, sub b) { \
//   T self = { a.x / b, a.y / b }; return self; }
// #define Vec2_Muls(T, sub) inline T T##_Muls(T a, sub b) { \
//   T self = { a.x * b, a.y * b }; return self; }
// #define Vec2_Subs(T, sub) inline T T##_Subs(T a, sub b) { \
//   T self = { a.x - b, a.y - b }; return self; }
//
// #define Vec2_IAdds(T, sub) inline void T##_IAdds(T* a, sub b) { \
//   a->x += b; a->y += b; }
// #define Vec2_IDivs(T, sub) inline void T##_IDivs(T* a, sub b) { \
//   a->x /= b; a->y /= b; }
// #define Vec2_IMuls(T, sub) inline void T##_IMuls(T* a, sub b) { \
//   a->x *= b; a->y *= b; }
// #define Vec2_ISubs(T, sub) inline void T##_ISubs(T* a, sub b) { \
//   a->x -= b; a->y -= b; }
//      */
//
// }