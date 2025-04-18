const sended_class_loaders = new Set();

function send_class_loader(cl) {
  const System = Java.use('java.lang.System');
  let cl_id = System.identityHashCode(cl);
  while (cl != null && !sended_class_loaders.has(cl_id)) {
    let parent_ = cl.getParent();
    send({"type": "classloader", "data": {
      "id": cl_id,
      "parent_id": System.identityHashCode(parent_),
      "str": cl.toString(),
      "cname": cl.$className
    }});
    sended_class_loaders.add(cl_id);
    cl = parent_;
  }
}

function dump_classloaders() {
  Java.perform(() => {
    var class_loader = Java.enumerateClassLoadersSync();
    for (var cl of class_loader) {
      send_class_loader(cl);
    }
    send({"type": "classloader-done"})
  });
}

/* ----- Frida Native Class Loading -----
 * Broken, for some ineffable frida-android reason.
function registerStackConsumer() {
  const Consumer = Java.use('java.util.function.Consumer');
  const Method = Java.use('java.lang.reflect.Method');
  const ArrayList = Java.use('java.util.ArrayList');
  const StackFrame = Java.use('java.lang.StackWalker$StackFrame');

  // Finding r8 optimized method for the Consumer interface
  let requiredMethods = Consumer.class.getDeclaredMethods();
  var lambdamethod = '';
  requiredMethods.forEach(m => {
    var meth = Java.cast(m, Method);
    let methodname = meth.getName();
    if (methodname.startsWith("$r8$lambda$")) {
      lambdamethod = '_' + methodname;
    };
  });

  let spec = {
    name: "TheseusAndroidStackConsumer",
    implements: [Consumer],
    fields: {
      stack: 'java.util.ArrayList',
    },
    methods: {
      '<init>': [{
        returnType: 'void',
        argumentTypes: [],
        implementation: function () {
          this.stack.value = ArrayList.$new();
        }
      }],
      'accept': function (frame) {
        var castedFrame = Java.cast(frame, StackFrame);
        this.stack.value.add(castedFrame);
      },
      'getStack': [{
        returnType: '[Ljava.lang.StackWalker$StackFrame;',
        argumentTypes: [],
        implementation: function () {
          return this.stack.value.toArray(Java.array('java.lang.StackWalker$StackFrame', []));
        },
      }],
      "andThen": [{
        returnType: 'java.util.function.Consumer',
        argumentTypes: ['java.util.function.Consumer'],
        implementation: function (cons) {
          return this.$super.andThen(cons);
        },
      }],
      "lambda$andThen$0": [{
        returnType: 'void',
        argumentTypes: ['java.util.function.Consumer', 'java.lang.Object'],
        implementation: function (consumer, obj) {},
      }],
      [lambdamethod]: [{
        returnType: 'void',
        argumentTypes: ['java.util.function.Consumer', 'java.util.function.Consumer', 'java.lang.Object'],
        implementation: function (cons1, cons2, obj) {}
      }]
    },
  };

  console.log(Object.keys(spec.methods));
  return Java.registerClass(spec);
}
*/

// ----- InMemoryDexClassLoader class loading -----
function registerStackConsumer() {
  const InMemoryDexClassLoader = Java.use("dalvik.system.InMemoryDexClassLoader");
  const ByteBuffer = Java.use("java.nio.ByteBuffer");
  const Base64 = Java.use("android.util.Base64");
  let myClassLoader = InMemoryDexClassLoader.$new(
    ByteBuffer.wrap(Base64.decode("<PYTHON REPLACE StackConsumer.dex.b64>", Base64.DEFAULT.value)),
    null,
  );
  return Java.ClassFactory.get(myClassLoader).use("theseus.android.StackConsumer");
}

// recv('dump-class-loaders', function onMessage(msg) {dump_classloaders()});

Java.perform(() => {

  /*
  //const StackFrameInfo = Java.use('java.lang.StackFrameInfo');
  const Consumer = Java.use('java.util.function.Consumer');
  const System = Java.use('java.lang.System');
  */

  const StackWalker = Java.use('java.lang.StackWalker');
  const StackWalkerOptions = Java.use('java.lang.StackWalker$Option');
  const StackWalkerOptionsShowHidden = StackWalkerOptions.valueOf("SHOW_HIDDEN_FRAMES");
  const StackWalkerOptionsShowReflect = StackWalkerOptions.valueOf("SHOW_REFLECT_FRAMES");
  const StackWalkerOptionsRetainClassReference = StackWalkerOptions.valueOf("RETAIN_CLASS_REFERENCE");
  const StackFrame = Java.use('java.lang.StackWalker$StackFrame');
  const ByteBuffer = Java.use("java.nio.ByteBuffer");
  const Base64 = Java.use("android.util.Base64");
  const Method = Java.use("java.lang.reflect.Method");
  const Class = Java.use("java.lang.Class");
  const Constructor = Java.use("java.lang.reflect.Constructor");
  const Modifier = Java.use("java.lang.reflect.Modifier");
  const DexFile = Java.use("dalvik.system.DexFile");
  const File = Java.use('java.io.File');
  const Files = Java.use('java.nio.file.Files');
  const Path = Java.use('java.nio.file.Path');
  const System = Java.use('java.lang.System');
  const Arrays = Java.use('java.util.Arrays');

  const StackConsumer = registerStackConsumer();

  const get_stack = function () {
    // console.log(Java.use("android.util.Log").getStackTraceString(Java.use("java.lang.Exception").$new()));
    //
    // TODO: use this instead? (https://developer.android.com/reference/java/lang/StackTraceElement)
    // Pro: - more robust (cf crash of maltoy app)
    //      - Works with any android version
    // Con: - Use java string: may require a lot of parsing
    //      - Use getLineNumber is iffy: returns either a line number when debug info are available or the dex address
    //        when no debug info. We prefere the address, but does this means we need to strip the apk before running?
    // var stack = Java.use("java.lang.Exception").$new().getStackTrace();
    // for (var i = 0; i < stack.length; i++) {
    //   console.log(stack[i].toString());
    // }
    // return [];
    var stackConsumer = StackConsumer.$new();
    var walker = StackWalker.getInstance(StackWalkerOptionsRetainClassReference);
    walker.forEach(stackConsumer);
    //walker.walk(stackConsumer.walkNFrame(20));
    var stack = stackConsumer.getStack()
    //send({"type": "stack", "data": stackConsumer.getStack()});
    return stack.map((frame) => {
      let cl = frame.getDeclaringClass().getClassLoader();
      send_class_loader(cl);
      return {
        "bytecode_index": frame.getByteCodeIndex(),
        "is_native": frame.isNativeMethod(),
	"method": frame.getDeclaringClass().descriptorString() + "->" +  frame.getMethodName() + frame.getDescriptor(),
        "cl_id": System.identityHashCode(cl),
	//{
          //"descriptor": frame.getDescriptor(),
          //"name": frame.getMethodName(),
          //"class": frame.getDeclaringClass().descriptorString(),
          // Broken for some reason
          //"args": frame.getMethodType().parameterArray().map((argty) => argty.getName()),
          //"ret": frame.getMethodType().returnType().getName(),
	//}
      };
    });
  };
  const get_method_dsc = function (mth) {
    // TODO: find a way to use MethodType (https://developer.android.com/reference/java/lang/invoke/MethodType)
    // MethodType.descriptorString()
    return mth.getDeclaringClass().descriptorString() +
      "->" +
      mth.getName() +
      "(" +
      mth.getParameterTypes().map((argty) => argty.descriptorString()).join('') +
      ")" +
      mth.getReturnType().descriptorString();
  };
  const get_constr_dsc = function (cnstr) {
    // TODO: find a way to use MethodType (https://developer.android.com/reference/java/lang/invoke/MethodType)
    // MethodType.descriptorString()
    return cnstr.getDeclaringClass().descriptorString() +
      "->" +
      "<init>" +
      "(" +
      cnstr.getParameterTypes().map((argty) => argty.descriptorString()).join('') +
      ")V";
  };


  // ****** Reflexive Method Calls ******

  // Method.invoke(obj, ..args)
  Method.invoke.overload(
    "java.lang.Object", "[Ljava.lang.Object;" // the Frida type parser is so cursted...
  ).implementation = function (obj, args) {
    let cl = this.getDeclaringClass().getClassLoader();
    send_class_loader(cl);
    send({
      "type": "invoke", 
	"data": {
          "method": get_method_dsc(this),
          "method_cl_id": System.identityHashCode(cl),
	  /*{
            "name": this.getName(),
            "class": this.getDeclaringClass().getName(),
            "args": this.getParameterTypes().map((argty) => argty.getName() ),
            "ret": this.getReturnType().getName(),
          },*/
	  "stack": get_stack(),
	  "is_static": Modifier.isStatic(this.getModifiers()),
      }
    });
    return this.invoke(obj, args);
  };

  // ****** Reflexive Class Instantiation ******

  // Class.newInstance()
  Class.newInstance.overload(
  ).implementation = function () {
    let cl = this.getClassLoader();
    send_class_loader(cl);
    send({
      "type": "class-new-inst", 
	"data": {
          "constructor": this.descriptorString() + "-><init>()V",
          "constructor_cl_id": System.identityHashCode(cl),
	  /*{
            "name": "<init>",
            "class": this.getName(),
            "args": [],
            "ret": "V",
          },*/
          "caller_method": "?",
          "addr": 0,
	  "stack": get_stack()
      }
    });
    return this.newInstance();
  };
  // Constructor.newInstance(..args)
  Constructor.newInstance.overload(
    "[Ljava.lang.Object;"
  ).implementation = function (args) {
    let cl = this.getDeclaringClass().getClassLoader();
    send_class_loader(cl);
    send({
      "type": "cnstr-new-isnt", 
	"data": {
          "constructor": get_constr_dsc(this),
          "constructor_cl_id": System.identityHashCode(cl),
	  /*
	  {
            "name": "<init>",
            "class": this.getDeclaringClass().getName(),
            "args": this.getParameterTypes().map((argty) => argty.getName()),
            "ret": "V",
          },
	  */
          "caller_method": "?",
          "addr": 0,
	  "stack": get_stack()
      }
    });
    return this.newInstance(args);
  };

  // ****** Dynamic Class Loading ******

  // DexFile.openDexFileNative(sourceName, outputName, flags, loader, elements): load .dex from file
  // See https://cs.android.com/android/platform/superproject/main/+/main:libcore/dalvik/src/main/java/dalvik/system/DexFile.java;drc=2f8a31e93fc238a88a48bfeed82557e07e1d5003;l=477
  // https://cs.android.com/android/platform/superproject/main/+/main:art/runtime/native/dalvik_system_DexFile.cc;drc=3d19fbcc09b1b44928639b06cd0b88f735cd988d;l=368
  DexFile.openDexFileNative.overload(
    'java.lang.String',
    'java.lang.String',
    'int',
    'java.lang.ClassLoader',
    '[Ldalvik.system.DexPathList$Element;',
  ).implementation = function (
    sourceName,
    outputName,
    flags,
    loader,
    elements,
  ) {
    let file = File.$new(sourceName);

    let path = Path.of(sourceName, []);
    let dex = Files.readAllBytes(path);
    let b64 = Base64.encodeToString(dex, Base64.DEFAULT.value);
    let classloader_class = null;
    let classloader_id = System.identityHashCode(loader);
    if (loader !== null) {
      send_class_loader(loader);
      classloader_class = loader.getClass().descriptorString();
    }
    send({
      "type": "load-dex",
      "data": {
        "dex": [b64],
        "classloader_class": classloader_class,
        "classloader": classloader_id,
        "classloader_parent": System.identityHashCode(loader.getParent()),
      }
    });

    let is_wr = file.canWrite();
    if (is_wr) {
      file.setReadOnly();
    }
    let result = this.openDexFileNative(
      sourceName,
      outputName,
      flags,
      loader,
      elements,
    );
    /* TODO: FIX
    if (is_wr) {
      file.setWritable(true, false);
    }
    */
    return result;
  };
  // DexFile.openInMemoryDexFilesNative(bufs, arrays, starts, ends, loader,elements): load .dex from memory
  // See https://cs.android.com/android/platform/superproject/main/+/main:libcore/dalvik/src/main/java/dalvik/system/DexFile.java;drc=2f8a31e93fc238a88a48bfeed82557e07e1d5003;l=431
  // https://cs.android.com/android/platform/superproject/main/+/main:art/runtime/native/dalvik_system_DexFile.cc;l=253;drc=3d19fbcc09b1b44928639b06cd0b88f735cd988d
  DexFile.openInMemoryDexFilesNative.overload(
    '[Ljava.nio.ByteBuffer;',
    '[[B',
    '[I',
    '[I',
    'java.lang.ClassLoader',
    '[Ldalvik.system.DexPathList$Element;',
  ).implementation = function (
    bufs,
    arrays,
    starts,
    ends,
    loader,
    elements,
  ) {
    let dex = [];
    // openInMemoryDexFilesNative() checks bufs.length == arrays.length == starts.length === ends.length
    for (let i = 0; i < bufs.length; i++) {
      let s = starts[i];
      let e = ends[i];
      // openInMemoryDexFilesNative() checks s < e
      let array = arrays[i];
      let buf = bufs[i];
      let raw = [];
      // match code from art/runtime/native/dalvik_system_DexFile.cc commit 3d19fbcc09b1b44928639b06cd0b88f735cd988d
      raw = Arrays.copyOf([], e-s);
      if (array === null) {
        raw = buf.get(s, raw, 0, e-s);
      } else {
        raw = Arrays.copyOfRange(array, s, e);
      }
      let b64 = Base64.encodeToString(raw, Base64.DEFAULT.value);
      dex.push(b64);
    }

    let classloader_class = "";
    let classloader_id = System.identityHashCode(loader);
    if (loader !== null) {
      classloader_class = loader.getClass().descriptorString();
      send_class_loader(loader);
    }
    send({
      "type": "load-dex",
      "data": {
        "dex": dex,
        "classloader_class": classloader_class,
        "classloader": classloader_id,
        "classloader_parent": System.identityHashCode(loader.getParent()),
      }
    });
    return this.openInMemoryDexFilesNative(
      bufs,
      arrays,
      starts,
      ends,
      loader,
      elements,
    );
  };
  dump_classloaders();
});

//recv('dump-class-loaders', function onMessage(msg) {dump_classloaders()});
