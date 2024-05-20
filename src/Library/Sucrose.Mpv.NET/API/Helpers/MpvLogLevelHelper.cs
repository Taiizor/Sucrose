namespace Sucrose.Mpv.NET.API
{
    public static class MpvLogLevelHelper
    {
        public static string GetStringForLogLevel(MpvLogLevel logLevel)
        {
            return logLevel switch
            {
                MpvLogLevel.None => "no",
                MpvLogLevel.Fatal => "fatal",
                MpvLogLevel.Error => "error",
                MpvLogLevel.Warning => "warn",
                MpvLogLevel.Info => "info",
                MpvLogLevel.V => "v",
                MpvLogLevel.Debug => "debug",
                MpvLogLevel.Trace => "trace",
                _ => null,
            };
        }
    }
}