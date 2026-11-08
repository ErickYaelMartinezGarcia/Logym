using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

namespace SecuGen.SecuSearchSDK3
{
    public class SecuSearch
    {
        private static bool is64Bit = IntPtr.Size == 8;
        private IntPtr hLib = IntPtr.Zero;

        public SecuSearch()
        {
            String dllName = is64Bit ?
                SSConstants.SECUSEARCH_API_DLL_NAME_64BIT :
                SSConstants.SECUSEARCH_API_DLL_NAME_32BIT;

            hLib = Win32.LoadLibrary(dllName);
        }

        ~SecuSearch()
        {
            if (hLib != IntPtr.Zero)
            {
                Win32.FreeLibrary(hLib);
            }
        }

        private bool isDllLoaded()
        {
            return hLib != IntPtr.Zero;
        }


        public SSError InitializeEngine(SSParam param)
        {
            if (!isDllLoaded())
                return SSError.SECUSEARCHAPI_DLL_UNLOADED;

            if (is64Bit)
                return SecuSearchAPI64.InitializeEngine(ref param);
            else
                return SecuSearchAPI32.InitializeEngine(ref param);
        }

        public SSError TerminateEngine()
        {
            if (!isDllLoaded())
                return SSError.SECUSEARCHAPI_DLL_UNLOADED;

            if (is64Bit)
                return SecuSearchAPI64.TerminateEngine();
            else
                return SecuSearchAPI32.TerminateEngine();
        }

        public SSError GetEngineParam(ref SSParam param)
        {
            if (!isDllLoaded())
                return SSError.SECUSEARCHAPI_DLL_UNLOADED;

            if (is64Bit)
                return SecuSearchAPI64.GetEngineParam(ref param);
            else
                return SecuSearchAPI32.GetEngineParam(ref param);
        }


        public SSError RegisterFP(Byte[] sgTemplate, UInt32 templateId)
        {
            if (!isDllLoaded())
                return SSError.SECUSEARCHAPI_DLL_UNLOADED;

            if (is64Bit)
                return SecuSearchAPI64.RegisterFP(sgTemplate, templateId);
            else
                return SecuSearchAPI32.RegisterFP(sgTemplate, templateId);
        }


        public SSError RegisterFPBatch(SSIdTemplatePair[] pairs, Int32 count)
        {
            if (!isDllLoaded())
                return SSError.SECUSEARCHAPI_DLL_UNLOADED;

            if (count < 0)
                return SSError.INVALID_PARAM;

            if (is64Bit)
                return SecuSearchAPI64.RegisterFPBatch(pairs, (UInt64)count);
            else
                return SecuSearchAPI32.RegisterFPBatch(pairs, (UInt32)count);
        }


        public SSError RemoveFP(UInt32 templateId)
        {
            if (!isDllLoaded())
                return SSError.SECUSEARCHAPI_DLL_UNLOADED;

            if (is64Bit)
                return SecuSearchAPI64.RemoveFP(templateId);
            else
                return SecuSearchAPI32.RemoveFP(templateId);
        }

        public SSError RemoveFPBatch(UInt32[] templateIds, Int32 count)
        {
            if (!isDllLoaded())
                return SSError.SECUSEARCHAPI_DLL_UNLOADED;

            if (is64Bit)
                return SecuSearchAPI64.RemoveFPBatch(templateIds, (UInt64)count);
            else
                return SecuSearchAPI32.RemoveFPBatch(templateIds, (UInt32)count);
        }

        public SSError SearchFP(Byte[] templateBuff, ref SSCandList candList)
        {
            if (!isDllLoaded())
                return SSError.SECUSEARCHAPI_DLL_UNLOADED;

            if (is64Bit)
                return SecuSearchAPI64.SearchFP(templateBuff, ref candList);
            else
                return SecuSearchAPI32.SearchFP(templateBuff, ref candList);
        }


        public SSError IdentifyFP(Byte[] sgTemplate, SSConfLevel seculevel, ref UInt32 templateId)
        {
            if (!isDllLoaded())
                return SSError.SECUSEARCHAPI_DLL_UNLOADED;

            if (is64Bit)
                return SecuSearchAPI64.IdentifyFP(sgTemplate, seculevel, ref templateId);
            else
                return SecuSearchAPI32.IdentifyFP(sgTemplate, seculevel, ref templateId);
        }


        public SSError SaveFPDB(String filename)
        {
            if (!isDllLoaded())
                return SSError.SECUSEARCHAPI_DLL_UNLOADED;

            if (is64Bit)
                return SecuSearchAPI64.SaveFPDB(filename);
            else
                return SecuSearchAPI32.SaveFPDB(filename);
        }


        public SSError LoadFPDB(String filename)
        {
            if (!isDllLoaded())
                return SSError.SECUSEARCHAPI_DLL_UNLOADED;

            if (is64Bit)
                return SecuSearchAPI64.LoadFPDB(filename);
            else
                return SecuSearchAPI32.LoadFPDB(filename);
        }


        public SSError ClearFPDB()
        {
            if (!isDllLoaded())
                return SSError.SECUSEARCHAPI_DLL_UNLOADED;

            if (is64Bit)
                return SecuSearchAPI64.ClearFPDB();
            else
                return SecuSearchAPI32.ClearFPDB();
        }


        public SSError GetFPCount(ref Int32 count)
        {
            if (!isDllLoaded())
                return SSError.SECUSEARCHAPI_DLL_UNLOADED;

            SSError err;

            if (is64Bit)
            {
                UInt64 count64 = 0;
                err = SecuSearchAPI64.GetFPCount(ref count64);
                count = (Int32)count64;
            }
            else
            {
                UInt32 count32 = 0;
                err = SecuSearchAPI32.GetFPCount(ref count32);
                count = (Int32)count32;
            }

            return err;
        }


        public SSError GetIDList(List<UInt32> idList)
        {
            return GetIDList(idList, Int32.MaxValue);
        }

        public SSError GetIDList(List<UInt32> idList, Int32 maxCount)
        {
            if (!isDllLoaded())
                return SSError.SECUSEARCHAPI_DLL_UNLOADED;

            idList.Clear();

            Int32 count = 0;
            SSError err = GetFPCount(ref count);

            if (err != SSError.NONE)
                return err;

            if (count <= 0 || maxCount <= 0)
                return SSError.NONE;

            count = Math.Min(count, maxCount);

            UInt32[] pList = new UInt32[count];

            err = GetIDList(pList, count, ref count);
            if (err != SSError.NONE)
                return err;

            for (int i = 0; i < count; i++)
                idList.Add(pList[i]);

            return SSError.NONE;
        }

        private SSError GetIDList(UInt32[] idList, Int32 maxCount, ref Int32 count)
        {
            SSError err;

            if (is64Bit)
            {
                UInt64 count64 = 0;
                err = SecuSearchAPI64.GetIDList(idList, (UInt64)maxCount, ref count64);
                count = (Int32)count64;
            }
            else
            {
                UInt32 count32 = 0;
                err = SecuSearchAPI32.GetIDList(idList, (UInt32)maxCount, ref count32);
                count = (Int32)count32;
            }

            return err;
        }

        public SSError GetTemplate(UInt32 templateId, Byte[] sgTemplate)
        {
            if (!isDllLoaded())
                return SSError.SECUSEARCHAPI_DLL_UNLOADED;

            if (is64Bit)
                return SecuSearchAPI64.GetTemplate(templateId, sgTemplate);
            else
                return SecuSearchAPI32.GetTemplate(templateId, sgTemplate);
        }


        public SSError ExtractTemplate(Byte[] standardTemplate, SSTemplateType templateType, UInt32 indexOfView, Byte[] sgTemplate)
        {
            if (!isDllLoaded())
                return SSError.SECUSEARCHAPI_DLL_UNLOADED;

            if (is64Bit)
                return SecuSearchAPI64.ExtractTemplate(standardTemplate, templateType, indexOfView, sgTemplate);
            else
                return SecuSearchAPI32.ExtractTemplate(standardTemplate, templateType, indexOfView, sgTemplate);
        }


        public SSError GetNumberOfView(Byte[] standardTemplate, SSTemplateType templateType, ref UInt32 numberOfView)
        {
            if (!isDllLoaded())
                return SSError.SECUSEARCHAPI_DLL_UNLOADED;

            SSError err;

            if (is64Bit)
            {
                UInt64 count64 = 0;
                err = SecuSearchAPI64.GetNumberOfView(standardTemplate, templateType, ref count64);
                numberOfView = (UInt32)count64;
            }
            else
            {
                UInt32 count32 = 0;
                err = SecuSearchAPI32.GetNumberOfView(standardTemplate, templateType, ref count32);
                numberOfView = (UInt32)count32;
            }

            return err;
        }


        public String GetVersion()
        {
            if (!isDllLoaded())
                return "";

            IntPtr pStr;

            if (is64Bit)
                pStr = SecuSearchAPI64.GetVersion();
            else
                pStr = SecuSearchAPI32.GetVersion();

            string str = Marshal.PtrToStringAnsi(pStr);
            pStr = IntPtr.Zero;

            return str;
        }
    }
}
