package com.secugen.secusearch.sample;

import com.secugen.secusearch.api.SSIdTemplatePair;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;

public class TestResourceUtils {
    private static final String templateDirName = "/min_data";
    private static final String ansiTemplateFilename = "/test_data/ansi378_2views.bin";
    private static final String isoTemplateFilename = "/test_data/iso19794_1view.bin";
    public static final int ANSI_TEMPLATE_VIEW_COUNT = 2;
    public static final int ISO_TEMPLATE_VIEW_COUNT = 1;

    /**
     * Reads all the fingerprint minutiae templates in 'min_data'.
     * Each template data is in the SecuGen 400-byte format and
     * each template ID is extracted from the first part of its file name.
     *
     * @return an arrays of SSIdTemplatePair
     * @throws IOException if it failed
     */
    public static SSIdTemplatePair[] readSecuGenTemplates() throws IOException {
        SSIdTemplatePair[] templates = new SSIdTemplatePair[51];

        for (int i = 0; i < templates.length; i++) {
            int id = 200000 + i;
            byte[] sgTemplate = readBinaryResourceFile(templateDirName + "/" + id + "_0_0.min");
            templates[i] = new SSIdTemplatePair(id, sgTemplate);
        }

        return templates;
    }

    /**
     * Reads a ANSI 378 template in 'test_data'.
     *
     * @return a byte array containing a ANSI 378 template
     * @throws IOException
     */
    public static byte[] readAnsiTemplate() throws IOException {
        return readBinaryResourceFile(ansiTemplateFilename);
    }

    /**
     * Reads a ISO 19794 template in 'test_data'.
     *
     * @return a byte array containing a ISO 19794 template
     * @throws IOException
     */
    public static byte[] readIsoTemplate() throws IOException {
        return readBinaryResourceFile(isoTemplateFilename);
    }

    /**
     * Reads a resource file.
     *
     * @param resourceFilename a resource file name to read
     * @return a byte array containing the binary data of the {@code resourceFilename}
     * @throws IOException if it failed
     */
    private static byte[] readBinaryResourceFile(String resourceFilename) throws IOException {
        InputStream in = TestResourceUtils.class.getResourceAsStream(resourceFilename);
        if (in == null) {
            throw new IOException("cannot open a file: " + resourceFilename);
        }
        ByteArrayOutputStream out = new ByteArrayOutputStream();
        byte[] buffer = new byte[4096];
        int len = in.read(buffer);
        while (len > 0) {
            out.write(buffer, 0, len);
            len = in.read(buffer);
        }
        in.close();
        out.close();
        return out.toByteArray();
    }

}
