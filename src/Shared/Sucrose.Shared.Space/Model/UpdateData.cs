namespace Sucrose.Shared.Space.Model
{
    internal class UpdateData(bool SilentMode, string AppVersion)
    {
        public bool SilentMode { get; set; } = SilentMode;

        public string AppVersion { get; set; } = AppVersion;
    }
}