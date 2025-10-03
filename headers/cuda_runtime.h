enum cudaError { _ = 0};
enum cudaChannelFormatKind { __ = 0 };

typedef enum cudaError cudaError_t;
typedef struct CUstream_st *cudaStream_t;
typedef struct cudaArray *cudaArray_t;
typedef struct cudaMipmappedArray *cudaMipmappedArray_t;
struct cudaPitchedPtr {
    void *ptr;
    size_t pitch;
    size_t xsize;
    size_t ysize;
};
struct cudaExtent {
    size_t width;
    size_t height;
    size_t depth;
};
struct cudaChannelFormatDesc {
    int x;
    int y;
    int z;
    int w;
    enum cudaChannelFormatKind f;
};