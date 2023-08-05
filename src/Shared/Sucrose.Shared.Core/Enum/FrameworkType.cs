using System.ComponentModel.DataAnnotations;

namespace Sucrose.Shared.Core.Enum
{
    internal enum FrameworkType
    {
        [DisplayAttribute(Name = "Unknown", Description = "Unknown")]
        Unknown,
        [DisplayAttribute(Name = ".NET 6.0", Description = ".NET_6.0")]
        NET_6_0,
        [DisplayAttribute(Name = ".NET 7.0", Description = ".NET_7.0")]
        NET_7_0,
        [DisplayAttribute(Name = ".NET 8.0", Description = ".NET_8.0")]
        NET_8_0,
        [DisplayAttribute(Name = ".NET Framework 4.8", Description = ".NET_Framework_4.8")]
        NET_Framework_4_8,
        [DisplayAttribute(Name = ".NET Framework 4.8.1", Description = ".NET_Framework_4.8.1")]
        NET_Framework_4_8_1
    }
}