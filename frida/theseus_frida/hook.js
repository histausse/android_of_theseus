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
  const Base64 = Java.use("android.util.Base64");
  const InMemoryDexClassLoader = Java.use("dalvik.system.InMemoryDexClassLoader");
  const ByteBuffer = Java.use("java.nio.ByteBuffer");
  const myClassLoader = InMemoryDexClassLoader.$new(
    ByteBuffer.wrap(Base64.decode("<PYTHON REPLACE StackConsumer.dex.b64>", Base64.DEFAULT.value)), 
    null
  );
  const StackConsumer = Java.ClassFactory.get(myClassLoader).use("theseus.android.StackConsumer");

  const get_stack = function () {
    var stackConsumer = StackConsumer.$new();
    var walker = StackWalker.getInstance(StackWalkerOptionsRetainClassReference);
    walker.forEach(stackConsumer);
    //send({"type": "stack", "data": stackConsumer.getStack()});
    return stackConsumer.getStack().map((frame) => {
      return {
        "bytecode_index": frame.getByteCodeIndex(),
        "is_native": frame.isNativeMethod(),
	"method": frame.getDeclaringClass().descriptorString() + "->" +  frame.getMethodName() + frame.getDescriptor(),
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

  const Method = Java.use("java.lang.reflect.Method");
  const Class = Java.use("java.lang.Class");
  const Constructor = Java.use("java.lang.reflect.Constructor");
  const Modifier = Java.use("java.lang.reflect.Modifier");
  Method.invoke.overload(
    "java.lang.Object", "[Ljava.lang.Object;" // the Frida type parser is so cursted...
  ).implementation = function (obj, args) {
    send({
      "type": "invoke", 
	"data": {
          "method": get_method_dsc(this),
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
  Class.newInstance.overload(
  ).implementation = function () {
    send({
      "type": "class-new-inst", 
	"data": {
          "constructor": this.descriptorString() + "-><init>()V",
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
  Constructor.newInstance.overload(
    "[Ljava.lang.Object;"
  ).implementation = function (args) {
    send({
      "type": "cnstr-new-isnt", 
	"data": {
          "constructor": get_constr_dsc(this),
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

});

