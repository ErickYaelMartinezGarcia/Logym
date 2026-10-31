#ifndef SECUSEARCH_ERR_HEADER
#define SECUSEARCH_ERR_HEADER

typedef int SSE_ERROR;

//Error message
#define FPS_ERROR_NONE                          0
#define FPS_ERROR_NOT_INIT                      1		
#define FPS_ERROR_INVALID_PARAM                 2
#define FPS_ERROR_SAVE_DB                       102
#define FPS_ERROR_LOAD_DB                       103
#define FPS_ERROR_INVALD_TEMPLATE               104
#define FPS_ERROR_DATA_EXIST                    105

#define FPS_ERROR_OVER_LIMIT                    201		
#define FPS_ERROR_IDENTIFICATION_FAIL           202		

#define FPS_ERROR_TOO_FEW_MINUTIAE              301
#define FPS_ERROR_TOO_FEW_FEAT                  302

#define FPS_ERROR_LICENSE_LOAD                  501
#define FPS_ERROR_LICENSE_KEY                   502
#define FPS_ERROR_LICENSE_EXPIRED               503
#define FPS_ERROR_LICENSE_WRITE                 504


// Memory
#define FPS_ERROR_HEAP_ALLOC                    2004
#define FPS_ERROR_HEAP_FREE                     2005
#define FPS_ERROR_SET_LOCK_PAGE_PRIVILEGE       2006
#define FPS_ERROR_ALLOC_PHYS_MEM                2007
#define FPS_ERROR_FREE_PHYS_MEM                 2008
#define FPS_ERROR_MAP_PHYS_MEM                  2009
#define FPS_ERROR_RESERVE_MEM                   2010
#define FPS_ERROR_LOW_MEM                       2011

#define FPS_ERROR_TOO_MANY_FEAT                 2101
#define FPS_ERROR_DATA_NOT_FOUND                2103

#endif
