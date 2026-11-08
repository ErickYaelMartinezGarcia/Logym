// SecuSearch.h : main header file for the error codes and definitions of the SecuSearch API
// Date			: 2017.03.09

#ifndef SECUSEARCH_DEF_HEADER
#define SECUSEARCH_DEF_HEADER

#include <stdint.h>
#include "SecuSearchErr.h"

#define CONF_LEVEL_LOWEST           1
#define CONF_LEVEL_LOWER            2
#define CONF_LEVEL_LOW              3
#define CONF_LEVEL_BELOW_NORMAL     4
#define CONF_LEVEL_NORMAL           5
#define CONF_LEVEL_ABOVE_NORMAL     6
#define CONF_LEVEL_HIGH             7
#define CONF_LEVEL_HIGHER           8	
#define CONF_LEVEL_HIGHEST          9

#define FIN_NUM_UNKNOWN             0
#define FIN_NUM_RIGHT_THUMB         1
#define FIN_NUM_RIGHT_INDEX         2
#define FIN_NUM_RIGHT_MIDDLE        3
#define FIN_NUM_RIGHT_RING          4
#define FIN_NUM_RIGHT_LITTLE        5
#define FIN_NUM_LEFT_THUMB          6
#define FIN_NUM_LEFT_INDEX          7
#define FIN_NUM_LEFT_MIDDLE         8
#define FIN_NUM_LEFT_RING           9
#define FIN_NUM_LEFT_LITTLE         10


#define SS_TEMPLATE_SG400           0
#define SS_TEMPLATE_FIR             1
#define SS_TEMPLATE_ANSI378         2
#define SS_TEMPLATE_ISO19794        3

#define SS_MAX_MATCH_SCORE          9999

#ifndef SG_TEMPLATE_SIZE
#define SG_TEMPLATE_SIZE            400
#endif

#define SSE_TEMPLATE_SIZE           SG_TEMPLATE_SIZE  

typedef uint32_t SSE_TEMPLATE_ID;

#define INVALID_SSE_TEMPLATE_ID     ((SSE_TEMPLATE_ID)(-1))

#define SSE_MAX_CANDIDATE_COUNT     300

typedef struct SSE_PARAM
{
    int32_t concurrency;
    int32_t candidateCount;
    char* szLicenseFile;
    int32_t bEnableRotation;
    uint8_t* licenseToken;
} SSE_PARAM;

typedef struct CANDIDATE
{
    SSE_TEMPLATE_ID id;
    int32_t matchScore;
    int32_t confidenceLevel;
} CANDIDATE;

typedef struct CAND_LIST
{
    struct CANDIDATE candidates[SSE_MAX_CANDIDATE_COUNT];
    int32_t count;
} CAND_LIST;

typedef struct ID_TEMPLATE_PAIR
{
    SSE_TEMPLATE_ID id;
    uint8_t templet[SSE_TEMPLATE_SIZE];
} ID_TEMPLATE_PAIR;

#ifdef _WIN64
#define SECUSEARCH_API_DLL_NAME	(TCHAR*)"secusearchapi.dll"
#else
#define SECUSEARCH_API_DLL_NAME	(TCHAR*)"secusearchapi32.dll"
#endif

#define SSE_API __cdecl

#ifdef SSE_DLL_EXPORTS
#define SSE_DLL __declspec(dllexport)
#else
#define SSE_DLL __declspec(dllimport)
#endif

typedef SSE_ERROR(SSE_API *FP_SecuSearch_InitializeEngine)(SSE_PARAM* param);
typedef SSE_ERROR(SSE_API *FP_SecuSearch_TerminateEngine)(void);
typedef SSE_ERROR(SSE_API *FP_SecuSearch_GetEngineParam)(SSE_PARAM* param);
typedef SSE_ERROR(SSE_API *FP_SecuSearch_RegisterFP)(uint8_t *sg_template, SSE_TEMPLATE_ID template_id);
typedef SSE_ERROR(SSE_API *FP_SecuSearch_RegisterFPBatch)(const ID_TEMPLATE_PAIR *pairs, size_t count);
typedef SSE_ERROR(SSE_API *FP_SecuSearch_RemoveFP)(SSE_TEMPLATE_ID template_id);
typedef SSE_ERROR(SSE_API *FP_SecuSearch_RemoveFPBatch)(SSE_TEMPLATE_ID *template_ids, size_t count);
typedef SSE_ERROR(SSE_API *FP_SecuSearch_SearchFP)(uint8_t *sg_template, CAND_LIST* candList);
typedef SSE_ERROR(SSE_API *FP_SecuSearch_IdentifyFP)(uint8_t *sg_template, uint32_t seculevel, SSE_TEMPLATE_ID *template_id);
typedef SSE_ERROR(SSE_API *FP_SecuSearch_LoadFPDB)(const char* filename);
typedef SSE_ERROR(SSE_API *FP_SecuSearch_SaveFPDB)(const char* filename);
typedef SSE_ERROR(SSE_API *FP_SecuSearch_ClearFPDB)();
typedef SSE_ERROR(SSE_API *FP_SecuSearch_GetFPCount)(uint32_t* count);
typedef SSE_ERROR(SSE_API *FP_SecuSearch_GetIDList)(SSE_TEMPLATE_ID* pIDList, uint32_t maxCount, uint32_t* count);
typedef SSE_ERROR(SSE_API *FP_SecuSearch_GetTemplate)(SSE_TEMPLATE_ID template_id, uint8_t* sg_template);
typedef SSE_ERROR(SSE_API *FP_SecuSearch_ExtractTemplate)(const uint8_t* standard_template, uint32_t template_type, uint32_t index_of_view, uint8_t* sg_template);
typedef SSE_ERROR(SSE_API *FP_SecuSearch_GetNumberOfView)(const uint8_t* standard_template, uint32_t template_type, size_t *number_of_view);
typedef const char* (SSE_API *FP_SecuSearch_GetVersion)(void);

#endif // SECUSEARCH_DEF_HEADER
