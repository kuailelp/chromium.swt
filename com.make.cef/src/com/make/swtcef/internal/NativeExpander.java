package com.make.swtcef.internal;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.channels.Channels;
import java.nio.channels.ReadableByteChannel;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Collections;
import java.util.Properties;

import org.eclipse.swt.internal.Platform;

public class NativeExpander {
	public static String expand() {
		Properties props = new Properties();
		new Detector().detect(props, Collections.<String>emptyList());
		String bundleFolder = props.get(Detector.DETECTED_NAME) + "-" + props.get(Detector.DETECTED_ARCH);

		System.out.println("bundleFolder: " + bundleFolder);
		String propsFile = "/" + bundleFolder + "/files.properties";

		try {
			InputStream files = NativeExpander.class.getResourceAsStream(propsFile);
			if (files == null) {
				throw new RuntimeException("Could not load " + propsFile);
			}
			props = new Properties();
			props.load(files);

			//System.out.println(props);

			String cefVersion = props.getProperty("cefVersion");
			Path cefPath = Paths.get(System.getProperty("user.home"), ".swtcef", cefVersion);
			System.out.println("swtcef path: " + cefPath);

			if (!Files.exists(cefPath)) {
				cefPath = Files.createDirectories(cefPath);
				for (String propName : props.stringPropertyNames()) {
					String filePath = props.getProperty(propName);
//					System.out.println(propName + ":" + filePath);
					if (!"cefVersion".equals(propName)) {
						InputStream is = NativeExpander.class.getResourceAsStream("/" + filePath);
						if (is == null) {
							throw new RuntimeException("Could not load " + "/" + filePath);
						}
						copy(cefPath, filePath, is);
					}
				}
				System.out.println("Expanded CEF natives to " + cefPath);
			}
			return cefPath.resolve(bundleFolder).toString();
		} catch (IOException e) {
			e.printStackTrace();
		}
		return "";
	}

	private static void copy(Path cefPath, String filePath, InputStream is) throws IOException {
		FileOutputStream os = null;

		try {
			Path pathFile = cefPath.resolve(filePath);
			Files.createDirectories(pathFile.getParent());
			File ex = pathFile.toFile();
			//ex.createNewFile();
//			System.out.println("Copying file to " + ex);
			os = new FileOutputStream(ex);
			ReadableByteChannel srcChannel = Channels.newChannel(is);

			for (long pos = 0L; is.available() > 0; pos += os.getChannel().transferFrom(srcChannel, pos,
					(long) Math.max(4096, is.available()))) {
				;
			}

			os.close();
			if (pathFile.endsWith("cefrust_subp")) {
				chmod ("755", ex.toString());
			}
			os = null;
		} finally {
			if (os != null) {
				os.close();
			}
			is.close();
		}
	}
	
	static void chmod(String permision, String path) {
		if (Platform.PLATFORM.equals ("win32")) return; //$NON-NLS-1$
		try {
			Runtime.getRuntime ().exec (new String []{"chmod", permision, path}).waitFor(); //$NON-NLS-1$
		} catch (Throwable e) {}
	}
}