#include "stdafx.h"
#include <windows.h>
#include "loadsse.h"

static HINSTANCE g_hSSELib = NULL;

//---------------------------------------------------------------------------
// load secusearchapi.dll
//---------------------------------------------------------------------------
SSELoadError LoadSSE(TCHAR* dllname, SecuSearchAPI** pLPSSE)
{
    *pLPSSE = NULL;

    g_hSSELib = LoadLibrary(dllname);
    if (!g_hSSELib)
        return DLL_ERR_LOAD;

    FP_SecuSearch_GetAPI SecuSearch_GetAPI = (FP_SecuSearch_GetAPI)GetProcAddress(g_hSSELib, "SecuSearch_GetAPI");

    if (SecuSearch_GetAPI)
    {
        *pLPSSE = SecuSearch_GetAPI();
        return DLL_ERR_NONE;
    } else
        return DLL_ERR_GET_PROC;
}

//---------------------------------------------------------------------------
// free secusearchapi.dll
//---------------------------------------------------------------------------
void FreeSSE()
{
    if (g_hSSELib)
    {
        FreeLibrary(g_hSSELib);
        g_hSSELib = NULL;
    }
}
