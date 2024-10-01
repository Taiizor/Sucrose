using System.ComponentModel.DataAnnotations;

namespace Sucrose.Shared.Core.Enum
{
    internal enum UpdateChannelType
    {
        [DisplayAttribute(Name = "Release", Description = "Release")]
        Release,
        [DisplayAttribute(Name = "PreRelease", Description = "Pre-Release")]
        PreRelease
    }

    internal enum UpdateExtensionType
    {
        [DisplayAttribute(Name = "7z", Description = ".7z")]
        Compressed,
        [DisplayAttribute(Name = "Exe", Description = ".exe")]
        Executable
    }
}