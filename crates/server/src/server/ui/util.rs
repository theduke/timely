use maud::{html, Render};

use crate::db::types::User;

use super::Fragment;

pub fn navbar(_ctx: &crate::server::Context, user: &User) -> Fragment {
    html! {
        nav class="navbar" role="navigation" aria-label="main navigation" {
          div class="navbar-brand" {
            a class="navbar-item" href="https://bulma.io" {
              // img src="https://bulma.io/images/bulma-logo.png" width="112" height="28" {}
              "Timely"
            }

            a role="button" class="navbar-burger" aria-label="menu" aria-expanded="false" data-target="navbarBasicExample" {
              span aria-hidden="true" {}
              span aria-hidden="true" {}
              span aria-hidden="true" {}
            }
          }

          div id="navbarBasicExample" class="navbar-menu" {
            div class="navbar-start" {
              a class="navbar-item" href="/" {
                "Dashboard"
              }

              // div class="navbar-item has-dropdown is-hoverable" {
              //   a class="navbar-link" {
              //     "More"
              //   }
              //   div class="navbar-dropdown" {
              //     a class="navbar-item" {
              //       "About"
              //     }
              //   }
              // }
            }

            div class="navbar-end" {
              div class="navbar-item" {
                div class="buttons" {
                  a.button {
                      (&user.username)
                  }

                  form action="/user/logout" method="post" {
                    button type="submit" class="button is-light" style="margin-bottom: 0;" {
                      "Log out"
                    }
                  }
                }
              }
            }
          }
        }
    }
}

pub fn renderiter<T, I>(iter: I) -> Fragment
where
    T: Render,
    I: IntoIterator<Item = T>,
{
    html! {
        @for item in iter {
            (item)
        }
    }
}
