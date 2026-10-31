#ifndef MDB_H
#define MDB_H

#define SIZE_FILENAME 28
#define SIZE_MAX_TEMPLATE 1000
#define SG_TEMPLATE_SIZE    400

typedef struct TagTEMPLATE{
    char Filename[SIZE_FILENAME];
    int Index;
    unsigned char TBuffer[SIZE_MAX_TEMPLATE];
} TEMPLATE;

class MDB
{
public:
    MDB();
    ~MDB();

    bool Load(const char *pszPathName);
    int Size() { return m_count; };
    TEMPLATE* operator[] (int index);
private:
    void Clear();
    void Add(TEMPLATE *pTemplate);
    void Create(int numTemplates);

private:
    TEMPLATE *m_pTemplate;
    int m_templateAllocNum;
    int m_count;
};

#endif MDB_H	// MDB_H