namespace Sucrose.Shared.Space.Model
{
    internal class UpdateData(bool Silent, string Version)
    {
        public bool Silent { get; set; } = Silent;

        public string Version { get; set; } = Version;
    }
}