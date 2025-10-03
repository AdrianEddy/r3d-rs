#if defined(__clang__) || _MSC_VER >= 1600
    #include <stdint.h>
#endif

typedef int8_t  cl_char;
typedef uint8_t  cl_uchar;
typedef int16_t cl_short;
typedef uint16_t cl_ushort;
typedef int32_t cl_int;
typedef uint32_t cl_uint;
typedef int64_t cl_long;
typedef uint64_t cl_ulong;

typedef uint16_t cl_half;
typedef float            cl_float;
typedef double           cl_double;
typedef struct _cl_platform_id *    cl_platform_id;
typedef struct _cl_device_id *      cl_device_id;
typedef struct _cl_context *        cl_context;
typedef struct _cl_command_queue *  cl_command_queue;
typedef struct _cl_mem *            cl_mem;
typedef struct _cl_program *        cl_program;
typedef struct _cl_kernel *         cl_kernel;
typedef struct _cl_event *          cl_event;
typedef struct _cl_sampler *        cl_sampler;
typedef intptr_t            cl_context_properties;
typedef cl_uint             cl_bool;
typedef cl_ulong            cl_bitfield;
typedef cl_ulong            cl_properties;
typedef cl_bitfield         cl_device_type;
typedef cl_uint             cl_platform_info;
typedef cl_uint             cl_device_info;
typedef cl_bitfield         cl_device_fp_config;
typedef cl_uint             cl_device_mem_cache_type;
typedef cl_uint             cl_device_local_mem_type;
typedef cl_bitfield         cl_device_exec_capabilities;
typedef cl_bitfield         cl_command_queue_properties;
typedef cl_uint             cl_addressing_mode;
typedef cl_bitfield         cl_mem_flags;
typedef cl_uint             cl_context_info;
typedef cl_uint             cl_image_info;
typedef cl_uint             cl_program_info;
typedef cl_uint             cl_program_build_info;
typedef cl_uint             cl_filter_mode;
typedef cl_uint             cl_kernel_work_group_info;
typedef cl_bitfield         cl_map_flags;
typedef cl_uint             cl_mem_info;
typedef cl_uint             cl_channel_order;
typedef cl_uint             cl_channel_type;
typedef struct _cl_image_format {
    cl_channel_order        image_channel_order;
    cl_channel_type         image_channel_data_type;
} cl_image_format;
