using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using SecuGen.SecuSearchSDK3;

namespace SecuGen.SecuSearch3Samples
{
    class SSEAPITest
    {
        public const string SEARCH_DB_PATH = "sample.tdb";

        public SecuSearch SSearch;
        public bool Initialized;
        public MDB Mdb;

        SSEAPITest()
        {
            SSearch = new SecuSearch();
            Mdb = new MDB();
            Initialized = false;
            InitSSE();
        }

        ~SSEAPITest()
        {
            DeinitSSE();
        }

        void InitSSE()
        {
            Initialized = false;

            SSParam param;

            // How many candiates as results of search
            param.CandidateCount = 10;

            // How many engines to be created internally.
            // If the concurrency is zero, SecuSearch automatically determines the total count of the CPU cores and use all the cores.
            param.Concurrency = 0;

            // license file - Ask SecuGen with your volume number which bin\VolNoReader.exe returns.
            param.LicenseFile = "./license.dat";

            // Whether or not to allow any amount of fingerprint rotations.
            param.EnableRotation = true;

            SSError error = SSearch.InitializeEngine(param);
            switch (error)
            {
                case SSError.NONE:
                    Initialized = true;
                    break;
                case SSError.SECUSEARCHAPI_DLL_UNLOADED:
                    if (IntPtr.Size == 8)
                        Console.WriteLine("{0} is not loaded.", SSConstants.SECUSEARCH_API_DLL_NAME_64BIT);
                    else
                        Console.WriteLine("{0} is not loaded.", SSConstants.SECUSEARCH_API_DLL_NAME_32BIT);
                    break;
                case SSError.SET_LOCK_PAGE_PRIVILEGE:
                    Console.WriteLine("Cannot enable the SE_LOCK_MEMORY privilege");
                    break;
                case SSError.LICENSE_LOAD:
                case SSError.LICENSE_KEY:
                case SSError.LICENSE_EXPIRED:
                    Console.WriteLine("License file({0}) is missing or not valid", param.LicenseFile);
                    break;
                default:
                    Console.WriteLine("failed to initialize SecuSearch(code = {0})", error);
                    break;
            }
        }

        void DeinitSSE()
        {
            if (Initialized)
            {
                SSearch.TerminateEngine();
                Initialized = false;
            }
        }

        // Read .min files
        void LoadMDB(string mdbPathName)
        {
            Debug.Assert(Initialized);

            Console.WriteLine("loading template files in {0}", mdbPathName);
            bool loaded = Mdb.Load(mdbPathName);
            Debug.Assert(loaded);

            Console.WriteLine("mdb size = {0}", Mdb.Size);
            Console.WriteLine("loading templates done");
        }

        // Load internal template db into secusearch
        void LoadDB()
        {
            Debug.Assert(Initialized);

            Console.WriteLine("reloading template db: {0}", SEARCH_DB_PATH);
            SSError error = SSearch.LoadFPDB(SEARCH_DB_PATH);
            Debug.Assert(error == SSError.NONE);

            int fpCount = 0;
            error = SSearch.GetFPCount(ref fpCount);
            Debug.Assert(error == SSError.NONE);

            Console.WriteLine(">>  fpcount = {0}", fpCount);
            Console.WriteLine("reloading templates db done");
        }

        // Add/Register templates to secusearch
        SSError RegisterMDB()
        {
            Console.WriteLine("Registering templates ...>>>");
            int registerCount = 0;
            SSError error = SSError.NONE;
            for (int i = 0; i < Mdb.Size; i++)
            {
                uint templateID = Mdb.GetTemplate(i).Index;
                byte[] templateBuff = Mdb.GetTemplate(i).TBuffer;
                error = SSearch.RegisterFP(templateBuff, templateID);
                if (error != SSError.NONE)
                {
                    break;
                }
                registerCount++;
            }

            int fpCount = 0;
            error = SSearch.GetFPCount(ref fpCount);
            Debug.Assert(error == SSError.NONE);

            Console.Write("Registering templates done ");
            Console.WriteLine(">>  fpcount = {0}\n", fpCount);

            return error;
        }

        // Add/Register templates in batches to secusearch
        // to improve the speed by using multi-cores
        SSError RegisterMDBBatch()
        {
            int fpCountBefore = 0, fpCount = 0;

            SSError error = SSearch.GetFPCount(ref fpCountBefore);
            Debug.Assert(error == SSError.NONE);

            Console.WriteLine("Registering templates in batches ...>>>");
            int registerCount = 0;
            const int batchCount = 1000;
            SSIdTemplatePair[] pairs = new SSIdTemplatePair[batchCount];
            int i = 0;
            while (i < Mdb.Size)
            {
                int k;
                for (k = 0; k < batchCount && i < Mdb.Size; k++, i++)
                {
                    pairs[k].Id = (UInt32)i;
                    pairs[k].Template = Mdb.GetTemplate(i).TBuffer;
                }
                error = SSearch.RegisterFPBatch(pairs, k);
                if (error != SSError.NONE)
                {
                    break;
                }
                registerCount += k;
            }


            error = SSearch.GetFPCount(ref fpCount);
            Debug.Assert(error == SSError.NONE);
            Debug.Assert(fpCount == fpCountBefore + registerCount);

            Console.Write("Registering templates in batches done ");
            Console.WriteLine(">>  fpcount = {0}\n", fpCount);

            return error;
        }

        void Test(bool bLoadDB = false, bool testRemove = false)
        {
            if (!Initialized)
                return;

            string mdbPathName = "../min_data";

            SSError error = SSError.NONE;

            // Read template files
            LoadMDB(mdbPathName);

            if (bLoadDB)
            {
                LoadDB();
            }
            else
            {
                // Register
                error = RegisterMDB();
                Debug.Assert(error == SSError.NONE);

                // Save templates of secusearch into files
                // When shutting down secusearch, you can save templates registered 
                // and then reload on start-up of secusearch by calling LoadFPDB().
                Console.WriteLine();
                Console.WriteLine("write DB to {0}", SEARCH_DB_PATH);

                error = SSearch.SaveFPDB(SEARCH_DB_PATH);
                Debug.Assert(error == SSError.NONE);
            }

            // how many templates in mdb
            int testCount = Mdb.Size;

            // how many templates in secusearch
            int fpCount = 0;
            error = SSearch.GetFPCount(ref fpCount);
            Debug.Assert(error == SSError.NONE);

            // List IDs of templates registered
            List<uint> idList = new List<uint>();
            error = SSearch.GetIDList(idList);
            Debug.Assert(error == SSError.NONE);
            Debug.Assert(idList.Count == fpCount);

            foreach (uint id in idList)
            {
                Console.WriteLine("id = {0}", id);
            }

            //
            // search test
            //
            if (testRemove)
            {
                // Remove templates
                for (uint i = 0; i < testCount; i++)
                {
                    uint templateID = i;
                    SSearch.RemoveFP(templateID);	// Delete one template in secusearch
                }
            }
            else
            {
                error = SSearch.ClearFPDB();		// Delete all templates in secusearch
                Debug.Assert(error == SSError.NONE);
                Console.Write("Template DB is cleared ");

                error = SSearch.GetFPCount(ref fpCount);
                Debug.Assert(error == SSError.NONE);
                Console.WriteLine(">>  fpcount = {0}", fpCount);
                Debug.Assert(fpCount == 0);
            }

            // search : the candidate count must be zero because secusearch has no templates.
            SSCandList candList = new SSCandList();

            for (int i = 0; i < testCount; i++)
            {
                uint templateID = (uint)i;
                error = SSearch.SearchFP(Mdb.GetTemplate(i).TBuffer, ref candList);
                if (error != SSError.NONE)
                {
                    Console.WriteLine("search failed error: {0}", error);
                }

                Console.Write("{0} : ", Mdb.GetTemplate(i).Filename);
                Console.Write("{0} : ", templateID);
                if (candList.Count > 0)
                {
                    Console.Write("{0}   ", candList.Candidates[0].Id);
                    Console.WriteLine("matchScore={0}", candList.Candidates[0].MatchScore);
                }
                else
                {
                    Console.WriteLine(" no candidate");
                }
            }

            // add/register
            if (bLoadDB)
            {
                LoadDB();
            }
            else
            {
                RegisterMDBBatch();
            }

            //// search : scores should be high such as 9999
            for (int i = 0; i < testCount; i++)
            {
                uint templateID = (uint)i;
                error = SSearch.SearchFP(Mdb.GetTemplate(i).TBuffer, ref candList);
                if (error != SSError.NONE)
                {
                    Console.WriteLine("search failed error: {0}", error);
                }

                Console.Write("{0} : ", Mdb.GetTemplate(i).Filename);
                Console.Write("{0} : ", templateID);
                if (candList.Count > 0)
                {
                    Console.Write("{0}   ", candList.Candidates[0].Id);
                    Console.WriteLine("matchScore={0}", candList.Candidates[0].MatchScore);
                }
                else
                {
                    Console.WriteLine(" no candidate");
                }
            }

            // ANSI 378 and ISO 19794 standard template conversion
            Console.WriteLine("\nRegister and search a ANSI 378 template");

            byte[] ansiTemplate = File.ReadAllBytes("../test_data/ansi378_2views.bin");
            byte[] sgTemplate = new byte[SSConstants.TEMPLATE_SIZE];
            uint numberOfViews = 0;

            SSearch.GetNumberOfView(ansiTemplate, SSTemplateType.ANSI378, ref numberOfViews);
            Debug.Assert(numberOfViews == 2);

            for (uint indexOfView = 0; indexOfView < numberOfViews; indexOfView++)
            {
                uint templateId = 1234560 + indexOfView;

                error = SSearch.ExtractTemplate(ansiTemplate, SSTemplateType.ANSI378, indexOfView, sgTemplate);
                if (error != SSError.NONE)
                {
                    Console.WriteLine("template extraction failed error: {0}", error);
                    continue;
                }

                error = SSearch.RegisterFP(sgTemplate, templateId);
                if (error != SSError.NONE)
                {
                    Console.WriteLine("search failed error: {0}", error);
                    continue;
                }

                Console.WriteLine("registered the {0}-th view", indexOfView);

                error = SSearch.SearchFP(sgTemplate, ref candList);
                if (error != SSError.NONE)
                {
                    Console.WriteLine("search failed error: {0}", error);
                    continue;
                }

                Console.WriteLine("searched the {0}-th view", indexOfView);
                Console.WriteLine("expected candidate: {0}", templateId);

                if (candList.Count > 0)
                {
                    Console.Write("candidate: {0}, ", candList.Candidates[0].Id);
                    Console.WriteLine("matchScore={0}", candList.Candidates[0].MatchScore);
                }
                else
                {
                    Console.WriteLine(" no candidate");
                }
            }

            Console.WriteLine("\nRegister and search a ISO 19794 template");

            byte[] isoTemplate = File.ReadAllBytes("../test_data/iso19794_1view.bin");

            SSearch.GetNumberOfView(isoTemplate, SSTemplateType.ISO19794, ref numberOfViews);
            Debug.Assert(numberOfViews == 1);

            for (uint indexOfView = 0; indexOfView < numberOfViews; indexOfView++)
            {
                uint templateId = 9876540 + indexOfView;

                error = SSearch.ExtractTemplate(isoTemplate, SSTemplateType.ISO19794, indexOfView, sgTemplate);
                if (error != SSError.NONE)
                {
                    Console.WriteLine("template extraction failed error: {0}", error);
                    continue;
                }

                error = SSearch.RegisterFP(sgTemplate, templateId);
                if (error != SSError.NONE)
                {
                    Console.WriteLine("search failed error: {0}", error);
                    continue;
                }

                Console.WriteLine("registered the {0}-th view", indexOfView);

                error = SSearch.SearchFP(sgTemplate, ref candList);
                if (error != SSError.NONE)
                {
                    Console.WriteLine("search failed error: {0}", error);
                    continue;
                }

                Console.WriteLine("searched the {0}-th view", indexOfView);
                Console.WriteLine("expected candidate: {0}", templateId);

                if (candList.Count > 0)
                {
                    Console.Write("candidate: {0}, ", candList.Candidates[0].Id);
                    Console.WriteLine("matchScore={0}", candList.Candidates[0].MatchScore);
                }
                else
                {
                    Console.WriteLine(" no candidate");
                }
            }

        }

        static void Main(string[] args)
        {
            bool loadDB = false;

            if (args.Length == 1 && args[0] == "--loaddb")
                loadDB = true;

            //
            // NOTE: 
            //
            Console.WriteLine("Note:");
            Console.WriteLine("The following must be checked before running this sample.");
            Console.WriteLine("Otherwise, it will not work properly.");
            Console.WriteLine("  -license file: for example, license.dat");
            Console.WriteLine("  -run as administrator");
            Console.WriteLine();
            Console.WriteLine("Press Enter key to keep running...");
            Console.ReadLine();

            SSEAPITest sseApiTest = new SSEAPITest();

            if (sseApiTest.Initialized)
            {
                // How to use the APIs
                sseApiTest.Test(loadDB);
            }

            Console.WriteLine("\nApplication is done. Press Enter key to close.\n");
            Console.ReadLine();
        }

    } // SSAPITest

} // SecuGen.SecuSearch3Samples
