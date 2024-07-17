using Sucrose.XamlAnimatedGif.Properties;
using System.Windows.Markup;

[assembly: XmlnsDefinition(XmlnsInfo.XmlNamespace, "Sucrose.XamlAnimatedGif")]
[assembly: XmlnsPrefix(XmlnsInfo.XmlNamespace, "gif")]

namespace Sucrose.XamlAnimatedGif.Properties
{
    static class XmlnsInfo
    {
        public const string XmlNamespace = "https://github.com/XamlAnimatedGif/XamlAnimatedGif";
    }
}