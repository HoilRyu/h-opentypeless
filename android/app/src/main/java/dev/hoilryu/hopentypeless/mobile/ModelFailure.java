package dev.hoilryu.hopentypeless.mobile;

/** Only a known binary compatibility failure disables a model until recheck. */
final class ModelFailure {
    static final String TRANSIENT="transient", COMPATIBILITY="compatibility", QUALITY="quality";
    static String classify(Throwable error){return error instanceof LinkageError?COMPATIBILITY:TRANSIENT;}
    static boolean blocks(String kind){return COMPATIBILITY.equals(kind);}
}
