package com.secugen.secusearch.sample;

import com.secugen.secusearch.api.*;

import java.io.IOException;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Arrays;

/**
 *
 */
public class SSApiTest {
    private static final Path templateDbPath = Paths.get("secusearch.tdb");
    private final SecuSearch secuSearch;
    private final SSIdTemplatePair[] ssIdTemplatePairs;
    private final byte[] ansiTemplate;
    private final byte[] isoTemplate;

    public static void main(String[] args) {
        // Get the singleton instance of SecuSearch.
        final SecuSearch secuSearch = SecuSearch.getInstance();

        // You can call SecuSearch.getVersion() before or after the initialization.
        System.out.println("API version: " + secuSearch.getVersion());
        System.out.println();

        try {
            SSApiTest test = new SSApiTest(secuSearch);

            test.initializeEngine();

            test.registerFP();
            test.getFPCount();
            test.getTemplate();
            test.getIDList();

            test.searchFP();
            test.identifyFP();

            test.removeFP();
            test.getFPCount();

            test.registerFPBatch();
            test.getFPCount();
            test.searchFP();
            test.identifyFP();
            test.saveFPDB();

            test.removeFPBatch();
            test.getFPCount();

            test.loadFPDB();
            test.getFPCount();

            test.searchFP();
            test.identifyFP();

            test.clearFPDB();
            test.getFPCount();

            test.extractTemplate();

            test.terminateEngine();
        } catch (IOException e) {
            System.out.println("A file operation has failed - " + e);
        } catch (SSException e) {
            System.out.println("A SecuSearch operation has failed - " + e);
        }
    }

    public SSApiTest(SecuSearch secuSearch) throws IOException {
        this.secuSearch = secuSearch;
        this.ssIdTemplatePairs = TestResourceUtils.readSecuGenTemplates();
        this.ansiTemplate = TestResourceUtils.readAnsiTemplate();
        this.isoTemplate = TestResourceUtils.readIsoTemplate();
    }

    public void initializeEngine() throws SSException {
        System.out.print("initializeEngine: processing... ");

        int concurrency = 0;
        int candidateCount = 20;
        String licenseFile = ""; // You can only register up to 1,000 templates without specifying a license file.
        boolean enableRotation = false;

        secuSearch.initializeEngine(new SSEngineParam(concurrency, candidateCount, licenseFile, enableRotation));
        System.out.println("done");
    }

    public void registerFP() throws SSException {
        System.out.print("registerFP: " + ssIdTemplatePairs.length + " templates... ");

        for (SSIdTemplatePair pair : ssIdTemplatePairs) {
            if (!secuSearch.registerFP(pair.getTemplate(), pair.getId())) {
                System.out.println();
                System.out.println("registerFP: " + pair.getId() + " already exist.");
                return;
            }
        }

        System.out.println("done");
    }

    public void registerFPBatch() throws SSException {
        System.out.print("registerFPBatch: " + ssIdTemplatePairs.length + " templates... ");

        if (!secuSearch.registerFPBatch(ssIdTemplatePairs)) {
            System.out.println();
            System.out.println("registerFPBatch: some of IDs already exist.");
            return;
        }

        System.out.println("done");
    }

    public void removeFP() throws SSException {
        System.out.print("removeFP: " + ssIdTemplatePairs.length + " templates... ");
        int removedCount = 0;

        for (SSIdTemplatePair pair : ssIdTemplatePairs) {
            if (secuSearch.removeFP(pair.getId())) {
                removedCount++;
            }
        }

        if (removedCount == ssIdTemplatePairs.length) {
            System.out.println("done");
        } else {
            int unregisteredIdCount = ssIdTemplatePairs.length - removedCount;
            System.out.println(
                    removedCount + " templates are only removed " +
                    "since " + unregisteredIdCount + " IDs are not registered before.");
        }
    }

    public void removeFPBatch() throws SSException {
        int[] idsToRemove = new int[ssIdTemplatePairs.length];
        for (int i = 0; i < ssIdTemplatePairs.length; i++) {
            idsToRemove[i] = ssIdTemplatePairs[i].getId();
        }

        System.out.print("removeFPBatch: " + ssIdTemplatePairs.length + " templates... ");

        secuSearch.removeFPBatch(idsToRemove);

        System.out.println("done");
    }

    public void searchFP() throws SSException {
        System.out.print("searchFP: " + ssIdTemplatePairs.length + " templates... ");

        for (SSIdTemplatePair pair : ssIdTemplatePairs) {
            SSCandidate[] candidates = secuSearch.searchFP(pair.getTemplate());
            if (candidates.length == 0) {
                System.out.println("failed");
                System.out.println("no candidate for the template(id: " + pair.getId() + ")");
                return;
            } else if (candidates[0].getId() != pair.getId()) {
                System.out.println("failed");
                System.out.println("the template(id: " + pair.getId() + ") is not top-ranked.");
                return;
            }
        }

        System.out.println("done");
    }

    public void identifyFP() throws SSException {
        System.out.print("identifyFP: " + ssIdTemplatePairs.length + " templates... ");

        for (SSIdTemplatePair pair : ssIdTemplatePairs) {
            int templateId = secuSearch.identifyFP(pair.getTemplate(), SSConfLevel.NORMAL);
            if (templateId == SecuSearch.INVALID_TEMPLATE_ID) {
                System.out.println("failed");
                System.out.println("identifyFP: non-identified ID for the template(id: " + pair.getId() + ")");
                return;
            }
        }

        System.out.println("done");
    }

    public void clearFPDB() throws SSException {
        System.out.print("clearFPDB: removing all... ");

        secuSearch.clearFPDB();

        System.out.println("done");
    }

    public void loadFPDB() throws SSException {
        System.out.print("loadFPDB: loading a file(" + templateDbPath.toString() + ")... ");

        if (!secuSearch.loadFPDB(templateDbPath.toString())) {
            System.out.println("failed");
            return;
        }

        System.out.println("done");
    }

    public void saveFPDB() throws SSException {
        System.out.print("saveFPDB: saving as a file(" + templateDbPath.toString() + ")... ");

        if (!secuSearch.saveFPDB(templateDbPath.toString())) {
            System.out.println("saveFPDB: failed");
            return;
        }

        System.out.println("done");
    }

    public void getFPCount() throws SSException {
        System.out.println("getFPCount: " + secuSearch.getFPCount());
    }

    public void getTemplate() throws SSException {
        System.out.print("saveFPDB: getting " + templateDbPath.toString() + " templates... ");

        for (SSIdTemplatePair pair : ssIdTemplatePairs) {
            byte[] sgTemplate = secuSearch.getTemplate(pair.getId());
            if (!Arrays.equals(sgTemplate, pair.getTemplate())) {
                System.out.println("failed");
                System.out.println("getTemplate: an incorrect template is returned.");
                return;
            }
        }

        System.out.println("done");
    }

    public void getIDList() throws SSException {
        System.out.print("getIDList: processing...");

        int[] allIDs = secuSearch.getIDList();
        assert allIDs.length == secuSearch.getFPCount();

        System.out.println("done");
    }

    public void extractTemplate() throws SSException {
        int viewCount;
        byte[] sgTemplate;

        System.out.print("extractTemplate: processing a ANSI 378 template... ");

        viewCount = secuSearch.getNumberOfView(ansiTemplate, SSTemplateType.ANSI378);
        assert viewCount == TestResourceUtils.ANSI_TEMPLATE_VIEW_COUNT;

        for (int i = 0; i < viewCount; i++) {
            sgTemplate = secuSearch.extractTemplate(ansiTemplate, SSTemplateType.ANSI378, i);
            assert sgTemplate.length == SecuSearch.TEMPLATE_SIZE;
        }

        System.out.println("done");

        System.out.print("extractTemplate: processing a ISO 19794 template... ");

        viewCount = secuSearch.getNumberOfView(isoTemplate, SSTemplateType.ISO19794);
        assert viewCount == TestResourceUtils.ISO_TEMPLATE_VIEW_COUNT;

        for (int i = 0; i < viewCount; i++) {
            sgTemplate = secuSearch.extractTemplate(isoTemplate, SSTemplateType.ISO19794, i);
            assert sgTemplate.length == SecuSearch.TEMPLATE_SIZE;
        }

        System.out.println("done");
    }

    public void terminateEngine() throws SSException {
        System.out.print("terminateEngine: processing... ");

        secuSearch.terminateEngine();

        System.out.println("done");
    }

}
