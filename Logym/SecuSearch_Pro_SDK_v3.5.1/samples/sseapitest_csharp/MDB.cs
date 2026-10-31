using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using System.IO;
using System.Text;
using System.Threading.Tasks;
using SecuGen.SecuSearchSDK3;

namespace SecuGen.SecuSearch3Samples
{
    struct Template
    {
        public string Filename;
        public uint Index;
        public byte[] TBuffer;

        public Template(string filename, uint index, byte[] tBuffer)
        {
            Filename = filename;
            Index = index;
            TBuffer = tBuffer;
        }
    }

    class MDB
    {
        private List<Template> Templates;
        public int Size { get { return Templates.Count; } }

        public MDB()
        {
            Templates = new List<Template>();
        }

        public bool Load(string pathname)
        {
            string[] filePaths = Directory.GetFiles(pathname, "*.min");
            uint i = 0;
            foreach (string path in filePaths)
            {
                Template t = new Template(path, i, File.ReadAllBytes(path));
                Add(t);
                i++;
            }

            return Size > 0;
        }

        public Template GetTemplate(int index)
        {
            return (Template)Templates[index];
        }

        private void Add(Template template)
        {
            Templates.Add(template);
        }

    }

} // SecuGen.SecuSearch3Samples
