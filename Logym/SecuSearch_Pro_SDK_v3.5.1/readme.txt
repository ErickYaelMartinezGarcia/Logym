# SecuGen SecuSearch 3 SDK 
## Version 3.5.1
### January 9, 2025
### README
================================================================
			
### Supported Platforms
======================
Microsoft Windows 10/Windows 11
Micrsoft Visual Studio 2022
.Net Framework v4.x for SecuSearch .Net

### Files Included in this Distribution
=======================================
readme.txt          		- this file
bin\
	msvcr110.dll		- VC 2022 runtime DLL which is required to load SecuSearchAPI.Net.dll
	secusearchapi.dll	- SecuSearch 64bit Engine DLL
	SecuSearchAPI.Net.dll	- SecuSearch .Net API DLL
	secusearchapi32.dll	- SecuSearch 32bit Engine DLL
	sseapitest_cpp.exe	- SecuSearch C++ 64bit sample application
	sseapitest_cpp32.exe	- SecuSearch C++ 32bit sample application
	sseapitest_csharp.exe	- SecuSearch C# sample application
	volnoreader.exe		- Volume number reader for licensing
inc\
	SecuSearchAPI.h		- SecuSearch SDK header file
	SecuSearchAPI.hpp	- SecuSearch SDK header file
	SecuSearchDef.h		- SecuSearch SDK header file
	SecuSearchErr.h		- SecuSearch SDK header file
min_data\
	Directory containing fingerprint template files used by sample application
samples \
	Visual Studio 2022 source code for sseapitest_sample.exe & sseapitest_csharp.exe
test_data\
	ANSI and ISO template test data 
SecuSearch 3 SDK Pro Manual (Windows) SG1-0005D-007.pdf - SecuSearch SDK documentation

### Installation
================
1. Unzip the SecuSearch distribution
2. Configure the Windows memory policy to set the lock page privilege.
   Check out the detailed instructions in the SDK manual.
3. Obtain a license file if you think that the application requires greater than 1,000 templates to be stored in DB.  Proceed as follows:
	Step 1)	Run 'VolNoReader.exe' in the bin directory of this distribution.
	Step 2) Save the volume number of your hard disk as displayed by this application.
	Step 3) Send an emall including the saved file to your reseller or SecuGen Corporation to request a license file.
	Step 4) Copy the license file (license.dat) into the bin directory of this distribution or the directory where your application is running.
hard disk where SecuSearch engine is running.
	Step 5) Specify the license file name in ENGINE_PARAM structure on calling InitializeEngine.

### Release Notes
=================
v3.0 - Initial release
v3.1 - .Net API support
v3.2 - Added support for 32bit Windows
       Maximum number of fingerprints supported on 32bit platforms is 100,000.
v3.2.2 - Fixed problem with SeacuSearch.NET running on IIS 
v3.2.3 - Fixed problem loading templates with small number of minutiae
v3.2.4 - Allowance of DB to have up to 1,000 templates w/o license file.  After 1,000 templates in DB, license file drives max templates.
v3.2.7 - Fix bug with template allowance limitation compared against the license file.
v3.2.8 - Prevent multiple of the same candidate being returned in result set.
v3.5.0 - Change of license mechanism from System Volume ID to Hash of Windows Instance (support for Docker and AWS instances). Upgrade to VS2022, and Move from ISO C++ 14 to ISO C++ 17 standards.	
v3.5.1 - Minor documentation changes.