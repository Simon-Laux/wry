use std::{cell::RefCell, path::PathBuf};

use objc2::{rc::Retained, MainThreadMarker};
#[cfg(target_vendor = "apple")]
use objc2_foundation::NSString;
use objc2_foundation::{NSError, NSURL};
#[cfg(target_vendor = "apple")]
use objc2_web_kit::{WKContentRuleList, WKContentRuleListStore};

use crate::Error;

pub struct ContentRuleListStore {
  #[cfg(target_vendor = "apple")]
  store: Retained<WKContentRuleListStore>,
}

impl ContentRuleListStore {
  pub fn get_default_store() -> Result<Self, Error> {
    #[cfg(target_vendor = "apple")]
    unsafe {
      let Some(mtm) = MainThreadMarker::new() else {
        return Err(Error::NotMainThread);
      };
      let Some(store) = WKContentRuleListStore::defaultStore(mtm) else {
        return Err(Error::FailedToGetDefaultContentRuleStore);
      };

      Ok(ContentRuleListStore { store })
    }
  }

  pub fn new(url: &str) -> Result<Self, Error> {
    #[cfg(target_vendor = "apple")]
    unsafe {
      let Some(mtm) = MainThreadMarker::new() else {
        return Err(Error::NotMainThread);
      };
      let url = NSURL::fileURLWithPath(&NSString::from_str(url));
      let Some(store) = WKContentRuleListStore::storeWithURL(Some(&url), mtm) else {
        return Err(Error::FailedToGetDefaultContentRuleStore);
      };

      Ok(ContentRuleListStore { store })
    }
  }

  pub fn add_from_string<Cb: FnOnce(crate::Result<*mut WKContentRuleList>) + Send + 'static>(
    self,
    identifier: Option<String>,
    encoded_content_rule_list: &str,
    cb: Cb,
  ) {
    #[cfg(target_vendor = "apple")]
    unsafe {
      let identifier = identifier.map(|identifier| NSString::from_str(&identifier));
      let cb = RefCell::new(Some(cb));
      let block = block2::RcBlock::new(
        move |rule_list: *mut WKContentRuleList, error: *mut NSError| {
          if error.is_null() {
            if let Some(cb) = cb.take() {
              cb(Ok(rule_list));
            }
          } else if let Some(cb) = cb.take() {
            let Some(err_ref) = error.as_ref() else {
              return cb(Err(Error::NSError("Failed to read error".to_string())));
            };
            let description = err_ref.localizedDescription();
            cb(Err(Error::NSError(description.to_string())));
          }
        },
      );

      self
        .store
        .compileContentRuleListForIdentifier_encodedContentRuleList_completionHandler(
          identifier.as_deref(),
          Some(&NSString::from_str(encoded_content_rule_list)),
          Some(&block),
        );
    }
  }
}
