#ifndef SECUSEARCH_API_H_HEADER_
#define SECUSEARCH_API_H_HEADER_

#include "SecuSearchDef.h"


#ifdef __cplusplus
extern "C" {
#endif

SSE_DLL SSE_ERROR SSE_API SecuSearch_InitializeEngine(const SSE_PARAM* param);

SSE_DLL SSE_ERROR SSE_API SecuSearch_TerminateEngine(void);

SSE_DLL SSE_ERROR SSE_API SecuSearch_GetEngineParam(SSE_PARAM* param);

SSE_DLL SSE_ERROR SSE_API SecuSearch_RegisterFP(const uint8_t *sg_template, SSE_TEMPLATE_ID template_id);

SSE_DLL SSE_ERROR SSE_API SecuSearch_RegisterFPBatch(const ID_TEMPLATE_PAIR *pairs, size_t count);

SSE_DLL SSE_ERROR SSE_API SecuSearch_RemoveFP(SSE_TEMPLATE_ID template_id);

SSE_DLL SSE_ERROR SSE_API SecuSearch_RemoveFPBatch(SSE_TEMPLATE_ID *template_ids, size_t count);

SSE_DLL SSE_ERROR SSE_API SecuSearch_SearchFP(const uint8_t *sg_template, CAND_LIST* candList);

SSE_DLL SSE_ERROR SSE_API SecuSearch_IdentifyFP(const uint8_t *sg_template, uint32_t seculevel, SSE_TEMPLATE_ID *template_id);

SSE_DLL SSE_ERROR SSE_API SecuSearch_LoadFPDB(const char* filename);

SSE_DLL SSE_ERROR SSE_API SecuSearch_SaveFPDB(const char* filename);

SSE_DLL SSE_ERROR SSE_API SecuSearch_ClearFPDB();

SSE_DLL SSE_ERROR SSE_API SecuSearch_GetFPCount(size_t* count);

SSE_DLL SSE_ERROR SSE_API SecuSearch_GetIDList(SSE_TEMPLATE_ID* pIDList, size_t maxCount, size_t* count);

SSE_DLL SSE_ERROR SSE_API SecuSearch_GetTemplate(SSE_TEMPLATE_ID template_id, uint8_t* sg_template);

SSE_DLL SSE_ERROR SSE_API SecuSearch_ExtractTemplate(const uint8_t* standard_template, uint32_t template_type, uint32_t index_of_view, uint8_t* sg_template);

SSE_DLL SSE_ERROR SSE_API SecuSearch_GetNumberOfView(const uint8_t* standard_template, uint32_t template_type, size_t *number_of_view);

SSE_DLL const char* SSE_API SecuSearch_GetVersion(void);

#ifdef __cplusplus
}
#endif

#endif // SECUSEARCH_API_H_HEADER_
