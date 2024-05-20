namespace Sucrose.Mpv.NET.Player
{
    internal static class KeepOpenHelper
    {
        public static string ToString(KeepOpen keepOpen)
        {
            return keepOpen switch
            {
                KeepOpen.Yes => "yes",
                KeepOpen.No => "no",
                KeepOpen.Always => "always",
                _ => throw new ArgumentException("Invalid enumeration value."),
            };
        }

        public static KeepOpen FromString(string keepOpenString)
        {
            return keepOpenString switch
            {
                "yes" => KeepOpen.Yes,
                "no" => KeepOpen.No,
                "always" => KeepOpen.Always,
                _ => throw new ArgumentException("Invalid value for \"keep-open\" property."),
            };
        }
    }
}