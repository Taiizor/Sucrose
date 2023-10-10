using System.ComponentModel.DataAnnotations;

namespace Sucrose.Shared.Core.Enum
{
    internal enum ChannelType
    {
        [DisplayAttribute(Name = "Release", Description = "Release")]
        Release,
        [DisplayAttribute(Name = "PreRelease", Description = "Pre-Release")]
        PreRelease
    }
}