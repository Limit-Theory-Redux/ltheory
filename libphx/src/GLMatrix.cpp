#include "GLMatrix.h"
#include "Matrix.h"
#include "MatrixDef.h"
#include "OpenGL.h"
#include "PhxMath.h"
#include "Vec3.h"
#include "Matrix.h"
#include "ArrayList.h"

/* NOTE : LoadMatrix expects column-major memory layout, but we use row-major,
 *        hence the need for transpositions when taking a Matrix*. */

// OpenGL 2.1 style matrix stack emulation.
enum class GLMatrixMode {
    ModelView,
    Projection
};

struct GLMatrixStack {
    ArrayList(Matrix*, modelview);
    ArrayList(Matrix*, projection);

    GLMatrixMode mode;

    size_t count() const {
      switch (mode) {
        case GLMatrixMode::ModelView:
          return ArrayList_GetSize(modelview);
        case GLMatrixMode::Projection:
          return ArrayList_GetSize(projection);
      }
    }

    void push() {
      Matrix* m = Matrix_Clone(*top());
      switch (mode) {
        case GLMatrixMode::ModelView:
          ArrayList_Append(modelview, m);
        case GLMatrixMode::Projection:
          ArrayList_Append(projection, m);
      }
    }

    void pop() {
      switch (mode) {
        case GLMatrixMode::ModelView:
          ArrayList_RemoveLast(modelview);
        case GLMatrixMode::Projection:
          ArrayList_RemoveLast(projection);
      }
    }

    Matrix** top() const {
      switch (mode) {
        case GLMatrixMode::ModelView:
          return ArrayList_GetLastPtr(modelview);
        case GLMatrixMode::Projection:
          return ArrayList_GetLastPtr(projection);
      }
    }

    // Ownership of 'm' is transferred to this GLMatrixStack.
    void load(Matrix* m) {
      Matrix** current = top();
      Matrix_Free(*current);
      *current = m;
    }

    // Ownership of 'm' is not transferred.
    void mult(Matrix const* m) {
      load(Matrix_Product(m, *top()));
    }
};

static GLMatrixStack stack;

void GLMatrix_Clear () {
  stack.load(Matrix_Identity());
}

void GLMatrix_Load (Matrix const* matrix) {
  stack.load(Matrix_Transpose(matrix));
}

void GLMatrix_LookAt (Vec3d const* eye, Vec3d const* at, Vec3d const* up) {
  Vec3d z = Vec3d_Normalize(Vec3d_Sub(*at, *eye));
  Vec3d x = Vec3d_Normalize(Vec3d_Cross(z, Vec3d_Normalize(*up)));
  Vec3d y = Vec3d_Cross(x, z);

  /* TODO : Yet another sign flip. Sigh. */
  double mArray[16] = {
      x.x, y.x, -z.x, 0,
      x.y, y.y, -z.y, 0,
      x.z, y.z, -z.z, 0,
      0, 0, 0, 1,
  };
  Matrix m;
  for (int i = 0; i < 16; ++i) {
    m.m[i] = (float)mArray[i];
  }

  stack.mult(&m);
  GLMatrix_Translate(-eye->x, -eye->y, -eye->z);
}

void GLMatrix_ModeP () {
  stack.mode = GLMatrixMode::Projection;
}

void GLMatrix_ModeWV () {
  stack.mode = GLMatrixMode::ModelView;
}

void GLMatrix_Mult (Matrix const* matrix) {
  Matrix* transposed = Matrix_Transpose(matrix);
  stack.mult(transposed);
  Matrix_Free(transposed);
}

void GLMatrix_Perspective (double fovy, double aspect, double z0, double z1) {
  double rads = Pi * fovy / 360.0;
  double cot = 1.0 / Tan(rads);
  double dz = z1 - z0;
  double nf = -2.0 * (z0 * z1) / dz;

  Matrix m = {{
    float(cot / aspect),   0,               0,    0,
    0,            float(cot),               0,    0,
    0,              0, float(-(z0 + z1) / dz), -1.0,
    0,              0,              float(nf),    0,
  }};
  stack.mult(&m);
}

void GLMatrix_Pop () {
  stack.pop();
}

void GLMatrix_Push () {
  stack.push();
}

void GLMatrix_PushClear () {
  stack.push();
  stack.load(Matrix_Identity());
}

Matrix* GLMatrix_Get () {
  return *stack.top();
}

void GLMatrix_RotateX (double angle) {
  Matrix* rotateX = Matrix_RotationX((float)angle);
  stack.mult(rotateX);
  Matrix_Free(rotateX);
}

void GLMatrix_RotateY (double angle) {
  Matrix* rotateY = Matrix_RotationY((float)angle);
  stack.mult(rotateY);
  Matrix_Free(rotateY);
}

void GLMatrix_RotateZ (double angle) {
  Matrix* rotateZ = Matrix_RotationZ((float)angle);
  stack.mult(rotateZ);
  Matrix_Free(rotateZ);
}

void GLMatrix_Scale (double x, double y, double z) {
  Matrix* scale = Matrix_Scaling((float)x, (float)y, (float)z);
  stack.mult(scale);
  Matrix_Free(scale);
}

void GLMatrix_Translate (double x, double y, double z) {
  Matrix* translate = Matrix_Translation((float)x, (float)y, (float)z);
  stack.mult(translate);
  Matrix_Free(translate);
}
