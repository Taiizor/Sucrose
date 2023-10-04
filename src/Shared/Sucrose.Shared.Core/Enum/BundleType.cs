using System.ComponentModel.DataAnnotations;

namespace Sucrose.Shared.Core.Enum
{
    internal enum BundleType
    {
        [DisplayAttribute(Name = "7z", Description = ".7z")]
        Compressed,
        [DisplayAttribute(Name = "Exe", Description = ".exe")]
        Executable
    }
}