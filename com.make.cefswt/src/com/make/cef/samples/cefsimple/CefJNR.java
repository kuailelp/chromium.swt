package com.make.cef.samples.cefsimple;

import cef.capi.CEF;
import cef.capi.CEF.App;
import cef.capi.CEF.MainArgs;
import cef.capi.CEF.Settings;
import jnr.ffi.LibraryLoader;
import jnr.ffi.Memory;
import jnr.ffi.NativeType;
import jnr.ffi.Pointer;

public class CefJNR {
//	public static interface Cef {
//		int cef_version_info(int entry);
//		
//		int CefExecuteProcess(Pointer args, Pointer application, Pointer windows_sandbox_info);
//		
//		
//	}
	
	public static void main(String[] args) {
//		Cef cef = LibraryLoader.create(Cef.class).load("cef");
//		Cef cef = LibraryLoader.create(Cef.class).load("cef");
//		System.out.println(cef);
//		
////		System.out.println(cef.cef_version_info(2));
//		System.out.println(cef.cef_version_info(4));
//		System.out.println(cef.cef_version_info(5));
		
//		Pointer mainArgs = cef.CefMainArgs(args.length, null);
//		System.out.println(mainArgs);
		
//		Pointer<CefMainArgs> main_args = Pointer.allocate(CefMainArgs.class);
		
		MainArgs main_args = new MainArgs();
		main_args.argc = args.length;
		main_args.argv = main_args.new Pointer();
//		main_args.argv.set(Memory.allocate(CEF.RUNTIME, NativeType.ADDRESS));
		// Cef applications have multiple sub-processes (render, plugin, GPU, etc)
		// that share the same executable. This function checks the command-line and,
		// if this is a sub-process, executes the appropriate logic.
		int exit_code = CEF.executeProcess(main_args, null, null);
		if (exit_code >= 0) {
			// The sub-process has completed so return here.
			throw new RuntimeException(exit_code + "");
		}

		  // Install xlib error handlers so that the application won't be terminated
		  // on non-fatal errors.
//		  XSetErrorHandler(XErrorHandlerImpl);
//		  XSetIOErrorHandler(XIOErrorHandlerImpl);

		// Specify Cef global settings here.
		Settings settings = null;

		  // SimpleApp implements application-level callbacks for the browser process.
		  // It will create the first browser instance in OnContextInitialized() after
		  // Cef has initialized.
//		  CefRefPtr<SimpleApp> app(new SimpleApp);

		App app = new App();
		app.get_browser_process_handler = null;
		// Initialize Cef for the browser process.
		CEF.initialize(main_args, settings, app, null);

		  // Run the Cef message loop. This will block until CefQuitMessageLoop() is
		  // called.
		CEF.runMessageLoop();

		// Shut down Cef.
		CEF.shutdown();
	}
}
