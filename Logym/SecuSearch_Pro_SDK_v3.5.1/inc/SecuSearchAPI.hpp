#ifndef SECUSEARCH_API_HPP_HEADER_
#define SECUSEARCH_API_HPP_HEADER_

#include "SecuSearchDef.h"


// SecuSearchAPI is a singlton class.
class SecuSearchAPI
{
public:

    virtual SSE_ERROR InitializeEngine(const SSE_PARAM* param) = 0;

    virtual SSE_ERROR TerminateEngine(void) = 0;

    virtual SSE_ERROR GetEngineParam(SSE_PARAM* param) = 0;

    virtual SSE_ERROR RegisterFP(const uint8_t* sg_template, SSE_TEMPLATE_ID template_id) = 0;

    virtual SSE_ERROR RegisterFPBatch(const ID_TEMPLATE_PAIR *pairs, size_t count) = 0;

    virtual SSE_ERROR RemoveFP(SSE_TEMPLATE_ID template_id) = 0;

    virtual SSE_ERROR RemoveFPBatch(SSE_TEMPLATE_ID *template_ids, size_t count) = 0;

    virtual SSE_ERROR SearchFP(const uint8_t* sg_template, CAND_LIST* candList) = 0;

    virtual SSE_ERROR IdentifyFP(const uint8_t* sg_template, uint32_t seculevel, SSE_TEMPLATE_ID* template_id) = 0;

    virtual SSE_ERROR SaveFPDB(const char* filename) = 0;

    virtual SSE_ERROR LoadFPDB(const char* filename) = 0;

    virtual SSE_ERROR ClearFPDB() = 0;

    virtual SSE_ERROR GetFPCount(size_t* count) = 0;

    virtual SSE_ERROR GetIDList(SSE_TEMPLATE_ID* pIDList, size_t maxCount, size_t* count) = 0;

    virtual SSE_ERROR GetTemplate(SSE_TEMPLATE_ID template_id, uint8_t* sg_template) = 0;

    virtual SSE_ERROR ExtractTemplate(const uint8_t* standard_template, uint32_t template_type, uint32_t index_of_view, uint8_t* sg_template) = 0;

    virtual SSE_ERROR GetNumberOfView(const uint8_t* standard_template, uint32_t template_type, size_t *number_of_view) = 0;

    virtual const char* GetVersion() = 0;
};

typedef SecuSearchAPI* (SSE_API *FP_SecuSearch_GetAPI)();

#ifdef __cplusplus
extern "C" {
#endif

SSE_DLL SecuSearchAPI* SSE_API SecuSearch_GetAPI();

#ifdef __cplusplus
}
#endif

#endif // SECUSEARCH_API_HPP_HEADER_
