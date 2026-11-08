//---------------------------------------------------------------------------
#ifndef LoadSGFPA_H
#define LoadSGFPA_H
#include <SecuSearchAPI.hpp>

//---------------------------------------------------------------------------
enum SSELoadError {
    DLL_ERR_NONE,
    DLL_ERR_LOAD,
    DLL_ERR_GET_PROC,
    DLL_ERR_MAX
};

SSELoadError LoadSSE(TCHAR* dllname, SecuSearchAPI** pSSE);
void  FreeSSE();

//extern SGFPA* g_SGFPA;

#endif
