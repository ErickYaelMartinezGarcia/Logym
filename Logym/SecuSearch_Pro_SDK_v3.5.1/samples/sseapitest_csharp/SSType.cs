using System;
using System.Runtime.InteropServices;

namespace SecuGen.SecuSearchSDK3
{
    public enum SSError : Int32
    {
        NONE                        = 0,
        NOT_INIT                    = 1,
        INVALID_PARAM               = 2,
        SAVE_DB                     = 102,
        ERROR_LOAD_DB               = 103,
        INVALD_TEMPLATE             = 104,
        DATA_EXIST                  = 105,

        OVER_LIMIT                  = 201,
        IDENTIFICATION_FAIL         = 202,

        TOO_FEW_MINUTIAE            = 301,
        TOO_FEW_FEAT                = 302,

        LICENSE_LOAD                = 501,
        LICENSE_KEY                 = 502,
        LICENSE_EXPIRED             = 503,
        LICENSE_WRITE               = 504,

        SECUSEARCHAPI_DLL_UNLOADED  = 602,

        // Memory
        HEAP_ALLOC                  = 2004,
        HEAP_FREE                   = 2005,
        SET_LOCK_PAGE_PRIVILEGE     = 2006,
        ALLOC_PHYS_MEM              = 2007,
        FREE_PHYS_MEM               = 2008,
        MAP_PHYS_MEM                = 2009,
        RESERVE_MEM                 = 2010,
        LOW_MEM                     = 2011,

        TOO_MANY_FEAT               = 2101,
        DATA_NOT_FOUND              = 2103,
    }

    // Confidence Level
    public enum SSConfLevel : Int32
    {
        INVALID       = 0,
        LOWEST        = 1,
        LOWER         = 2,
        LOW           = 3,	
        BELOW_NORMAL  = 4,
        NORMAL        = 5,
        ABOVE_NORMAL  = 6,
        HIGH          = 7,
        HIGHER        = 8,
        HIGHEST       = 9,
    }

    // FingerNumber
    public enum SSFingerNumber : Int32
    {
        UNKNOWN       = 0,
        RIGHT_THUMB   = 1,
        RIGHT_INDEX   = 2,
        RIGHT_MIDDLE  = 3,
        RIGHT_RING    = 4,
        RIGHT_LITTLE  = 5,
        LEFT_THUMB    = 6,
        LEFT_INDEX    = 7,
        LEFT_MIDDLE   = 8,
        LEFT_RING     = 9, 
        LEFT_LITTLE   = 10,
    }

    public enum SSTemplateType : Int32
    {
        SG400         = 0,
        FIR           = 1,
        ANSI378       = 2,
        ISO19794      = 3
    }

    public struct SSConstants
    {
        public const UInt32 INVALID_TEMPLATE_ID = 0xFFFFFFFF;
        public const Int32 MAX_MATCH_SCORE = 9999;
        public const Int32 TEMPLATE_SIZE = 400;
        public const Int32 MAX_CANDIDATE_COUNT = 300;
        public const String SECUSEARCH_API_DLL_NAME_64BIT = "secusearchapi.dll";
        public const String SECUSEARCH_API_DLL_NAME_32BIT = "secusearchapi32.dll";
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Ansi)]
    public struct SSParam
    {
        public Int32 Concurrency;
        public Int32 CandidateCount;
        [MarshalAs(UnmanagedType.LPStr)]
        public String LicenseFile;
        [MarshalAs(UnmanagedType.I1)]
        public Boolean EnableRotation;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct SSCandidate
    {
        public UInt32 Id;
        public Int32 MatchScore;
        [MarshalAs(UnmanagedType.I4)]
        public SSConfLevel ConfidenceLevel;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct SSCandList
    {
        [MarshalAs(UnmanagedType.ByValArray, ArraySubType = UnmanagedType.Struct, SizeConst = SSConstants.MAX_CANDIDATE_COUNT)]
        public SSCandidate[] Candidates;
        public Int32 Count;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct SSIdTemplatePair
    {
        public UInt32 Id;
        [MarshalAs(UnmanagedType.ByValArray, ArraySubType = UnmanagedType.U1, SizeConst = SSConstants.TEMPLATE_SIZE)]
        public Byte[] Template;
    }

}
