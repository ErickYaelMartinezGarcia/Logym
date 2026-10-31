using System;
using System.Runtime.InteropServices;

namespace SecuGen.SecuSearchSDK3
{
    internal class SecuSearchAPI32
    {
        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_InitializeEngine", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError InitializeEngine(ref SSParam param);

        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_TerminateEngine", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError TerminateEngine();

        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_GetEngineParam", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError GetEngineParam(ref SSParam param);

        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_RegisterFP", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError RegisterFP(Byte[] sgTemplate, UInt32 templateId);

        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_RegisterFPBatch", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError RegisterFPBatch(SSIdTemplatePair[] pairs, UInt32 count);

        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_RemoveFP", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError RemoveFP(UInt32 templateId);

        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_RemoveFPBatch", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError RemoveFPBatch(UInt32[] templateIds, UInt32 count);

        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_SearchFP", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError SearchFP(Byte[] sgTemplate, ref SSCandList candList);

        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_IdentifyFP", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError IdentifyFP(Byte[] sgTemplate, SSConfLevel seculevel, ref UInt32 templateId);

        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_SaveFPDB", CharSet = CharSet.Ansi, CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError SaveFPDB(String filename);

        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_LoadFPDB", CharSet = CharSet.Ansi, CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError LoadFPDB(String filename);

        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_ClearFPDB", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError ClearFPDB();

        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_GetFPCount", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError GetFPCount(ref UInt32 count);

        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_GetIDList", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError GetIDList(UInt32[] idList, UInt32 maxCount, ref UInt32 count);

        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_GetTemplate", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError GetTemplate(UInt32 templateId, Byte[] sgTemplate);

        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_ExtractTemplate", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError ExtractTemplate(Byte[] standardTemplate, SSTemplateType templateType, UInt32 indexOfView, Byte[] sgTemplate);

        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_GetNumberOfView", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError GetNumberOfView(Byte[] standardTemplate, SSTemplateType templateType, ref UInt32 numberOfView);

        [DllImport("secusearchapi32.dll", EntryPoint = "SecuSearch_GetVersion", CharSet = CharSet.Ansi, CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr GetVersion();
    }

    internal class SecuSearchAPI64
    {
        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_InitializeEngine", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError InitializeEngine(ref SSParam param);

        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_TerminateEngine", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError TerminateEngine();

        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_GetEngineParam", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError GetEngineParam(ref SSParam param);

        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_RegisterFP", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError RegisterFP(Byte[] sgTemplate, UInt32 templateId);

        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_RegisterFPBatch", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError RegisterFPBatch(SSIdTemplatePair[] pairs, UInt64 count);

        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_RemoveFP", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError RemoveFP(UInt32 templateId);

        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_RemoveFPBatch", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError RemoveFPBatch(UInt32[] templateIds, UInt64 count);

        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_SearchFP", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError SearchFP(Byte[] sgTemplate, ref SSCandList candList);

        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_IdentifyFP", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError IdentifyFP(Byte[] sgTemplate, SSConfLevel seculevel, ref UInt32 templateId);

        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_SaveFPDB", CharSet = CharSet.Ansi, CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError SaveFPDB(String filename);

        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_LoadFPDB", CharSet = CharSet.Ansi, CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError LoadFPDB(String filename);

        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_ClearFPDB", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError ClearFPDB();

        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_GetFPCount", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError GetFPCount(ref UInt64 count);

        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_GetIDList", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError GetIDList(UInt32[] idList, UInt64 maxCount, ref UInt64 count);

        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_GetTemplate", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError GetTemplate(UInt32 templateId, Byte[] sgTemplate);

        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_ExtractTemplate", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError ExtractTemplate(Byte[] standardTemplate, SSTemplateType templateType, UInt32 indexOfView, Byte[] sgTemplate);

        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_GetNumberOfView", CallingConvention = CallingConvention.Cdecl)]
        public static extern SSError GetNumberOfView(Byte[] standardTemplate, SSTemplateType templateType, ref UInt64 numberOfView);

        [DllImport("secusearchapi.dll", EntryPoint = "SecuSearch_GetVersion", CharSet = CharSet.Ansi, CallingConvention = CallingConvention.Cdecl)]
        public static extern IntPtr GetVersion();
    }
}