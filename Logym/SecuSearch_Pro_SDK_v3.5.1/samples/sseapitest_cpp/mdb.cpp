#include "stdafx.h"
#include <Windows.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <tchar.h>
#include <iostream>
#include "mdb.h"

MDB::MDB() : 
    m_pTemplate(NULL)
{
    Clear();
}

MDB::~MDB()
{
    Clear();
}

void MDB::Clear()
{
    if (m_pTemplate)
        free(m_pTemplate);
    m_pTemplate = NULL;
    m_templateAllocNum = 0;
    m_count = 0;
}

void MDB::Create(int numTemplates) 
{
    Clear();

    m_templateAllocNum = numTemplates;
    m_pTemplate = (TEMPLATE*)malloc(sizeof(TEMPLATE) * numTemplates);
    std::cout << "total db size = " << numTemplates << std::endl;
}

bool MDB::Load(const char *pszPathName)
{
    WIN32_FIND_DATA find_data = {0};
    int numFiles = 0;
    char szPathName[_MAX_PATH] = {0};
    FILE *fp = NULL;
    TEMPLATE minTemplate = {0};

    // how many .min file
    _stprintf(szPathName, "%s\\*.min", pszPathName);
    HANDLE h = FindFirstFile(szPathName, &find_data);
    while (h != INVALID_HANDLE_VALUE) {
        numFiles++;

        if (!FindNextFile(h, &find_data))
            break;
    }
    FindClose(h);

    // Allocate memory
    Create(numFiles);

    // Add templates
    numFiles = 0;
    h = FindFirstFile(szPathName, &find_data);
    while (h != INVALID_HANDLE_VALUE) {
        _stprintf(szPathName, "%s\\%s", pszPathName, find_data.cFileName);
        fp = fopen(szPathName, "rb");
        if (fp) {
            std::cout << numFiles + 1 << " adding " << find_data.cFileName << std::endl;
            fread(minTemplate.TBuffer, SG_TEMPLATE_SIZE, 1, fp);
            _tcscpy(minTemplate.Filename, find_data.cFileName);
            minTemplate.Index = numFiles++;

            Add(&minTemplate);
            fclose(fp);
        }

        if (!FindNextFile(h, &find_data))
            break;
    }
    FindClose(h);

    return (numFiles > 0);
}

void MDB::Add(TEMPLATE *pTemplate)
{
    memcpy(m_pTemplate[m_count].Filename, pTemplate->Filename, SIZE_FILENAME);
    m_pTemplate[m_count].Index = pTemplate->Index;
    memcpy(m_pTemplate[m_count].TBuffer, pTemplate->TBuffer, SIZE_MAX_TEMPLATE);
    m_count++;
}

TEMPLATE* MDB::operator[] (int index)
{
    if (index < m_count)
        return &m_pTemplate[index];
    else
        return NULL;
}