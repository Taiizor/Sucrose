using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Data;

namespace Rhythm.Helpers;
public class BooleanToVisibilityConverter : IValueConverter
{
    public BooleanToVisibilityConverter()
    {
    }

    public object Convert(object value, Type targetType, object parameter, string language)
    {
        if (value is bool booleanValue)
        {
            if (parameter is string invert && invert == "Invert")
            {
                return booleanValue ? Visibility.Collapsed : Visibility.Visible;
            }
            return booleanValue ? Visibility.Visible : Visibility.Collapsed;
        }
        throw new ArgumentException("ExceptionBooleanToVisibilityConverterValueMustBeBoolean");
    }

    public object ConvertBack(object value, Type targetType, object parameter, string language)
    {
        if (value is Visibility visibilityValue)
        {
            if (parameter is string invert && invert == "Invert")
            {
                return visibilityValue == Visibility.Collapsed;
            }
            return visibilityValue == Visibility.Visible;
        }
        throw new ArgumentException("ExceptionBooleanToVisibilityConverterParameterMustBeVisibility");
    }

}
