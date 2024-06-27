namespace Sucrose.Shared.Space.Model
{
    internal class UpdateData(string Version)
    {
        public string Version { get; set; } = Version;
    }
}