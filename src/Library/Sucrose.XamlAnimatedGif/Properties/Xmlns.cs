using System.Windows.Markup;
using Sucrose.XamlAnimatedGif.Properties;

[assembly: XmlnsDefinition(XmlnsInfo.XmlNamespace, "Sucrose.XamlAnimatedGif")]
[assembly: XmlnsPrefix(XmlnsInfo.XmlNamespace, "gif")]

namespace Sucrose.XamlAnimatedGif.Properties
{
    static class XmlnsInfo
    {
        public const string XmlNamespace = "https://github.com/XamlAnimatedGif/XamlAnimatedGif";
    }
}