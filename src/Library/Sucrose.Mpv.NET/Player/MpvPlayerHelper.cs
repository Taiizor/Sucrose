namespace Sucrose.Mpv.NET.Player
{
    internal static class MpvPlayerHelper
    {
        public static string BoolToYesNo(bool yesNoBool)
        {
            return yesNoBool ? "yes" : "no";
        }

        public static bool YesNoToBool(string yesNoString)
        {
            return yesNoString == "yes";
        }
    }
}