namespace Sucrose.Mpv.NET.Player
{
    internal static class LoadMethodHelper
    {
        public static string ToString(LoadMethod loadMethod)
        {
            return loadMethod switch
            {
                LoadMethod.Replace => "replace",
                LoadMethod.Append => "append",
                LoadMethod.AppendPlay => "append-play",
                _ => throw new ArgumentException("Invalid enumeration value."),
            };
        }
    }
}