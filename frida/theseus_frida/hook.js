Java.perform(() => {

  const Method = Java.use("java.lang.reflect.Method");
  const Class = Java.use("java.lang.Class");
  const Constructor = Java.use("java.lang.reflect.Constructor");
  Method.invoke.overload(
    "java.lang.Object", "[Ljava.lang.Object;" // the Frida type parser is so cursted...
  ).implementation = function (obj, args) {
    send({
      "type": "invoke", 
	"data": {
          "method": {
            "name": this.getName(),
            "class": this.getDeclaringClass().getName(),
            "args": this.getParameterTypes().map((argty) => argty.getName() ),
            "ret": this.getReturnType().getName(),
          },
          "caller_method": "?",
          "addr": 0,
      }
    });
    return this.invoke(obj, args);
  };
  Class.newInstance.overload(
  ).implementation = function () {
    send({
      "type": "class-new-inst", 
	"data": {
          "constructor": {
            "name": "<init>",
            "class": this.getName(),
            "args": [],
            "ret": "V",
          },
          "caller_method": "?",
          "addr": 0,
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
          "constructor": {
            "name": "<init>",
            "class": this.getDeclaringClass().getName(),
            "args": this.getParameterTypes().map((argty) => argty.getName()),
            "ret": "V",
          },
          "caller_method": "?",
          "addr": 0,
      }
    });
    return this.newInstance(args);
  };


});

